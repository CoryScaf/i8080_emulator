mod disassemble;
mod emulate8080;
mod i8080;
mod shaders;

use std::env;
use std::fs;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;

use vulkano::buffer::Subbuffer;
use vulkano::{
    buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage},
    command_buffer::{
        allocator::StandardCommandBufferAllocator, AutoCommandBufferBuilder, CommandBufferUsage,
        CopyBufferToImageInfo, PrimaryCommandBufferAbstract, RenderPassBeginInfo,
    },
    descriptor_set::{
        allocator::StandardDescriptorSetAllocator, PersistentDescriptorSet, WriteDescriptorSet,
    },
    device::{
        physical::{PhysicalDevice, PhysicalDeviceType},
        Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo, QueueFlags,
    },
    format::Format,
    image::{
        sampler::{Filter, Sampler, SamplerAddressMode, SamplerCreateInfo},
        view::ImageView,
        Image, ImageCreateInfo, ImageType, ImageUsage,
    },
    instance::{Instance, InstanceCreateInfo},
    memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator},
    pipeline::{
        graphics::{
            color_blend::{AttachmentBlend, ColorBlendAttachmentState, ColorBlendState},
            input_assembly::{InputAssemblyState, PrimitiveTopology},
            multisample::MultisampleState,
            rasterization::RasterizationState,
            vertex_input::{Vertex, VertexDefinition},
            viewport::{Viewport, ViewportState},
            GraphicsPipelineCreateInfo,
        },
        layout::PipelineDescriptorSetLayoutCreateInfo,
        DynamicState, GraphicsPipeline, Pipeline, PipelineBindPoint, PipelineLayout,
        PipelineShaderStageCreateInfo,
    },
    render_pass::{Framebuffer, FramebufferCreateInfo, RenderPass, Subpass},
    swapchain::{
        acquire_next_image, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo,
    },
    sync::{self, GpuFuture},
    DeviceSize, Validated, VulkanError, VulkanLibrary,
};
use winit::event::ElementState;
use winit::platform::modifier_supplement::KeyEventExtModifierSupplement;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

use crate::emulate8080::copy_screen_memory;
use crate::emulate8080::run_emulation;

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Vertex2D {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],

    #[name("tex_coords")]
    #[format(R32G32_SFLOAT)]
    tex_coords: [f32; 2],
}

fn adjust_window_size(
    images: &[Arc<Image>],
    render_pass: Arc<RenderPass>,
    viewport: &mut Viewport,
) -> Vec<Arc<Framebuffer>> {
    let extent = images[0].extent();
    viewport.extent = [extent[0] as f32, extent[1] as f32];

    images
        .iter()
        .map(|image| {
            let view = ImageView::new_default(image.clone()).unwrap();
            Framebuffer::new(
                render_pass.clone(),
                FramebufferCreateInfo {
                    attachments: vec![view],
                    ..Default::default()
                },
            )
            .unwrap()
        })
        .collect::<Vec<_>>()
}

fn select_physical_device(
    instance: &Arc<Instance>,
    surface: &Arc<Surface>,
    device_extensions: &DeviceExtensions,
) -> (Arc<PhysicalDevice>, u32) {
    instance
        .enumerate_physical_devices()
        .unwrap()
        .filter(|pd| pd.supported_extensions().contains(&device_extensions))
        .filter_map(|pd| {
            pd.queue_family_properties()
                .iter()
                .enumerate()
                .position(|(i, q)| {
                    q.queue_flags.contains(QueueFlags::GRAPHICS)
                        && pd.surface_support(i as u32, &surface).unwrap_or(false)
                })
                .map(|q| (pd, q as u32))
        })
        .min_by_key(|(pd, _)| match pd.properties().device_type {
            PhysicalDeviceType::DiscreteGpu => 0,
            PhysicalDeviceType::IntegratedGpu => 1,
            PhysicalDeviceType::VirtualGpu => 2,
            PhysicalDeviceType::Cpu => 3,
            _ => 4,
        })
        .expect("No suitable device available")
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mut filename = String::new();
    let mut arg_iterator = 1;
    let mut do_test = false;
    let mut do_help = false;
    let mut do_dissassemble = false;

    // Get flags
    while arg_iterator < args.len() {
        match args[arg_iterator].as_str() {
            "-d" | "--disassemble" => do_dissassemble = true,
            "-f" | "--file" => {
                arg_iterator += 1;
                filename = args[arg_iterator].clone();
            }
            "-t" | "--test" => do_test = true,
            "-h" | "--help" => do_help = true,
            _ => panic!("Unknown flag given {}", args[arg_iterator]),
        }
        arg_iterator += 1;
    }

    // Print help info (Exits if help flag set)
    if do_help {
        println!("8080 Emulator");
        println!("flags                 input               description");
        println!("-d, --disassemble                         Disassemble file");
        println!("-f, --file            <filename>          Enter filename");
        println!("-t, --test                                Indicates test file");
        println!("-h, --help                                print command info");
        return;
    }

    // Get filename if it wasn't set in the flags
    if filename == "" {
        println!("File not provided (use -h or --help for flags)");
        println!("Enter ROM filename:");
        io::stdin()
            .read_line(&mut filename)
            .expect("Filename not entered.");
        filename = filename.trim().to_string();
    }

    // Read the filename to a buffer
    let mut buffer: Vec<u8> = match fs::read(filename.clone()) {
        Ok(res) => res,
        Err(why) => panic!("Failed to open file {}: {}", filename, why),
    };

    // Disassemble provided file
    if do_dissassemble {
        let mut program_counter = 0;
        while program_counter < buffer.len() {
            program_counter += disassemble::disassemble8080_op(&buffer, program_counter);
        }
        return;
    }

    // 8080 memory size is 2^64
    buffer.resize(0x10000, 0);

    let state = Arc::new(Mutex::new(i8080::State::new(buffer, do_test)));

    {
        let mut state = state.lock().unwrap();
        // Test files don't need vulkan
        if state.testing {
            // provided test needs to start at 0x100
            state.memory.rotate_right(0x100);

            // jump to 0x100
            state.memory[0x00] = 0xc3;
            state.memory[0x01] = 0x00;
            state.memory[0x02] = 0x01;

            // fix stack pointer location since starts at 0x100
            state.memory[0x170] = 0x07;

            // change 0x05 to ret since it is a print call
            state.memory[0x05] = 0xc9; // make sure it returns

            // start testing loop which adds special calls
            while !state.should_exit {
                state.check_and_print_call();
                let prev_pc = state.program_counter;
                emulate8080::emulate8080_op(&mut state);
                if state.program_counter == 0 {
                    println!("Exit from {:04x}", prev_pc);
                    state.should_exit = true;
                }
            }
            return;
        }
    }

    let event_loop = EventLoop::new().unwrap();

    let library = VulkanLibrary::new().expect("No local Vulkan library/DLL");
    let required_extensions = Surface::required_extensions(&event_loop);

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    )
    .expect("Failed to create instance");

    let window = Arc::new(
        WindowBuilder::new()
            .with_title("I8080 Emulator")
            .build(&event_loop)
            .unwrap(),
    );

    let surface = Surface::from_window(instance.clone(), window.clone()).unwrap();

    let device_extensions = DeviceExtensions {
        khr_swapchain: true,
        ..DeviceExtensions::empty()
    };

    let (phys_dev, queue_family_index) =
        select_physical_device(&instance, &surface, &device_extensions);

    println!(
        "Using device: {} (type: {:?})",
        phys_dev.properties().device_name,
        phys_dev.properties().device_type
    );

    let (device, mut queues) = Device::new(
        phys_dev,
        DeviceCreateInfo {
            queue_create_infos: vec![QueueCreateInfo {
                queue_family_index,
                ..Default::default()
            }],
            enabled_extensions: device_extensions,
            ..Default::default()
        },
    )
    .expect("Failed to create device");

    let queue = queues.next().unwrap();

    let (mut swapchain, images) = {
        let surface_capabilities = device
            .physical_device()
            .surface_capabilities(&surface, Default::default())
            .unwrap();

        let image_format = device
            .physical_device()
            .surface_formats(&surface, Default::default())
            .unwrap()[0]
            .0;

        Swapchain::new(
            device.clone(),
            surface,
            SwapchainCreateInfo {
                min_image_count: surface_capabilities.min_image_count.max(2),
                image_format,
                image_extent: window.inner_size().into(),
                image_usage: ImageUsage::COLOR_ATTACHMENT,
                composite_alpha: surface_capabilities
                    .supported_composite_alpha
                    .into_iter()
                    .next()
                    .unwrap(),
                ..Default::default()
            },
        )
        .unwrap()
    };

    let memory_allocator = Arc::new(StandardMemoryAllocator::new_default(device.clone()));

    let vertices = [
        Vertex2D {
            position: [-1.0, -1.0],
            tex_coords: [1.0, 0.0],
        },
        Vertex2D {
            position: [-1.0, 1.0],
            tex_coords: [0.0, 0.0],
        },
        Vertex2D {
            position: [1.0, -1.0],
            tex_coords: [1.0, 1.0],
        },
        Vertex2D {
            position: [1.0, 1.0],
            tex_coords: [0.0, 1.0],
        },
    ];

    let vertex_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::VERTEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        vertices,
    )
    .unwrap();

    let index_buffer = Buffer::from_iter(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::INDEX_BUFFER,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        vec![1, 0, 2, 1, 2, 3u16],
    )
    .unwrap();

    let render_pass = vulkano::single_pass_renderpass!(
        device.clone(),
        attachments: {
            color: {
                format: swapchain.image_format(),
                samples: 1,
                load_op: Clear,
                store_op: Store,
            },
        },
        pass: {
            color: [color],
            depth_stencil: {},
        },
    )
    .unwrap();

    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());

    let descriptor_set_allocator = Arc::new(StandardDescriptorSetAllocator::new(
        device.clone(),
        Default::default(),
    ));

    let upload_buffer: Subbuffer<[u8]> = Buffer::new_slice(
        memory_allocator.clone(),
        BufferCreateInfo {
            usage: BufferUsage::TRANSFER_SRC,
            ..Default::default()
        },
        AllocationCreateInfo {
            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
            ..Default::default()
        },
        (256 * 224 * 4) as DeviceSize,
    )
    .unwrap();

    let image = Image::new(
        memory_allocator.clone(),
        ImageCreateInfo {
            image_type: ImageType::Dim2d,
            format: Format::R8G8B8A8_SRGB,
            extent: [256, 224, 1],
            usage: ImageUsage::TRANSFER_DST | ImageUsage::SAMPLED,
            ..Default::default()
        },
        AllocationCreateInfo::default(),
    )
    .unwrap();

    let texture = ImageView::new_default(image.clone()).unwrap();

    let sampler = Sampler::new(
        device.clone(),
        SamplerCreateInfo {
            mag_filter: Filter::Nearest,
            min_filter: Filter::Nearest,
            address_mode: [SamplerAddressMode::Repeat; 3],
            ..Default::default()
        },
    )
    .unwrap();

    let pipeline = {
        let vs = shaders::vs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let fs = shaders::fs::load(device.clone())
            .unwrap()
            .entry_point("main")
            .unwrap();
        let vertex_input_state = Vertex2D::per_vertex()
            .definition(&vs.info().input_interface)
            .unwrap();
        let stages = [
            PipelineShaderStageCreateInfo::new(vs),
            PipelineShaderStageCreateInfo::new(fs),
        ];
        let layout = PipelineLayout::new(
            device.clone(),
            PipelineDescriptorSetLayoutCreateInfo::from_stages(&stages)
                .into_pipeline_layout_create_info(device.clone())
                .unwrap(),
        )
        .unwrap();
        let subpass = Subpass::from(render_pass.clone(), 0).unwrap();

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState {
                    topology: PrimitiveTopology::TriangleStrip,
                    ..Default::default()
                }),
                viewport_state: Some(ViewportState::default()),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.num_color_attachments(),
                    ColorBlendAttachmentState {
                        blend: Some(AttachmentBlend::alpha()),
                        ..Default::default()
                    },
                )),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap()
    };

    let layout = &pipeline.layout().set_layouts()[0];
    let set = PersistentDescriptorSet::new(
        &descriptor_set_allocator,
        layout.clone(),
        [
            WriteDescriptorSet::sampler(0, sampler),
            WriteDescriptorSet::image_view(1, texture),
        ],
        [],
    )
    .unwrap();

    let mut viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [0.0, 0.0],
        depth_range: 0.0..=1.0,
    };

    let mut framebuffers = adjust_window_size(&images, render_pass.clone(), &mut viewport);

    let mut uploads = AutoCommandBufferBuilder::primary(
        &command_buffer_allocator,
        queue.queue_family_index(),
        CommandBufferUsage::OneTimeSubmit,
    )
    .unwrap();

    copy_screen_memory(&state, &upload_buffer);

    uploads
        .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
            upload_buffer,
            image.clone(),
        ))
        .unwrap();

    let mut recreate_swapchain = false;
    let mut previous_frame_end = Some(
        uploads
            .build()
            .unwrap()
            .execute(queue.clone())
            .unwrap()
            .boxed(),
    );

    let thread_state = Arc::clone(&state);

    let handle = thread::spawn(move || {
        run_emulation(thread_state);
    });

    event_loop
        .run(move |event, elwt| {
            elwt.set_control_flow(ControlFlow::Poll);

            match event {
                Event::WindowEvent {
                    event: WindowEvent::CloseRequested,
                    ..
                } => {
                    {
                        let mut state = state.lock().unwrap();
                        state.should_exit = true;
                    }

                    elwt.exit();
                }
                Event::WindowEvent {
                    event: WindowEvent::Resized(_),
                    ..
                } => {
                    recreate_swapchain = true;
                }
                Event::WindowEvent {
                    event: WindowEvent::KeyboardInput { event, .. },
                    ..
                } => {
                    if event.state == ElementState::Pressed && !event.repeat {
                        match event.key_without_modifiers().as_ref() {
                            // Debug Utils
                            winit::keyboard::Key::Character("u") => {
                                let mut state = state.lock().unwrap();

                                state.stop_debug_stepping();
                            }
                            winit::keyboard::Key::Character("p") => {
                                let mut state = state.lock().unwrap();

                                state.start_debug_stepping();
                            }
                            winit::keyboard::Key::Character("n") => {
                                let mut state = state.lock().unwrap();

                                state.step_count += 1;
                            }
                            winit::keyboard::Key::Character("m") => {
                                let mut state = state.lock().unwrap();

                                state.step_count += 10;
                            }
                            winit::keyboard::Key::Character(",") => {
                                let mut state = state.lock().unwrap();

                                state.step_count += 100;
                            }
                            winit::keyboard::Key::Character(".") => {
                                let mut state = state.lock().unwrap();

                                state.step_count += 1000;
                            }
                            winit::keyboard::Key::Character("h") => {
                                let mut state = state.lock().unwrap();

                                state.call_interrupt(1);
                            }
                            winit::keyboard::Key::Character("b") => {
                                let state = state.lock().unwrap();

                                println!("--------------------------------------------------");
                                println!("| A|F |  | B|C |  | D|E |  | H|L |  | PC |  | SP |");
                                println!("|{:02x}|{:02x}|  |{:02x}|{:02x}|  |{:02x}|{:02x}|  |{:02x}|{:02x}|  |{:04x}|  |{:04x}|", state.reg_a, state.flags_to_u8(), state.reg_b, state.reg_c, state.reg_d, state.reg_e, state.reg_h, state.reg_l, state.program_counter, state.stack_pointer);
                                println!("--------------------------------------------------");
                            }
                            // Game inputs
                            winit::keyboard::Key::Character("c") => {
                                let mut state = state.lock().unwrap();

                                // deposit credit
                                state.in_ports[1] |= 0b00000001;
                            }
                            winit::keyboard::Key::Character("1") => {
                                let mut state = state.lock().unwrap();

                                // 1 player start
                                state.in_ports[1] |= 0b00000100;
                            }
                            winit::keyboard::Key::Character("2") => {
                                let mut state = state.lock().unwrap();

                                // 2 player start
                                state.in_ports[1] |= 0b00000010;
                            }
                            winit::keyboard::Key::Character("w") => {
                                let mut state = state.lock().unwrap();

                                // player 1 shoot
                                state.in_ports[1] |= 0b00010000;
                            }
                            winit::keyboard::Key::Character("a") => {
                                let mut state = state.lock().unwrap();

                                // player 1 left
                                state.in_ports[1] |= 0b00100000;
                            }
                            winit::keyboard::Key::Character("d") => {
                                let mut state = state.lock().unwrap();

                                // player 1 right
                                state.in_ports[1] |= 0b01000000;
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                                let mut state = state.lock().unwrap();

                                // player 2 shoot
                                state.in_ports[2] |= 0b00010000;
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft) => {
                                let mut state = state.lock().unwrap();

                                // player 2 left
                                state.in_ports[2] |= 0b00100000;
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight) => {
                                let mut state = state.lock().unwrap();

                                // player 2 right
                                state.in_ports[2] |= 0b01000000;
                            }
                            _ => (),
                        }
                    }
                    else if event.state == ElementState::Released && !event.repeat {
                        match event.key_without_modifiers().as_ref() {
                            // Game inputs
                            winit::keyboard::Key::Character("c") => {
                                let mut state = state.lock().unwrap();

                                // deposit credit
                                state.in_ports[1] &= 0b11111110;
                            }
                            winit::keyboard::Key::Character("1") => {
                                let mut state = state.lock().unwrap();

                                // 1 player start
                                state.in_ports[1] &= 0b11111011;
                            }
                            winit::keyboard::Key::Character("2") => {
                                let mut state = state.lock().unwrap();

                                // 2 player start
                                state.in_ports[1] &= 0b11111101;
                            }
                            winit::keyboard::Key::Character("w") => {
                                let mut state = state.lock().unwrap();

                                // player 1 shoot
                                state.in_ports[1] &= 0b11101111;
                            }
                            winit::keyboard::Key::Character("a") => {
                                let mut state = state.lock().unwrap();

                                // player 1 left
                                state.in_ports[1] &= 0b11011111;
                            }
                            winit::keyboard::Key::Character("d") => {
                                let mut state = state.lock().unwrap();

                                // player 1 right
                                state.in_ports[1] &= 0b10111111;
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp) => {
                                let mut state = state.lock().unwrap();

                                // player 2 shoot
                                state.in_ports[2] &= 0b11101111;
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft) => {
                                let mut state = state.lock().unwrap();

                                // player 2 left
                                state.in_ports[2] &= 0b11011111;
                            }
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight) => {
                                let mut state = state.lock().unwrap();

                                // player 2 right
                                state.in_ports[2] &= 0b10111111;
                            }
                            _ => (),
                        }
                    }
                }
                Event::WindowEvent {
                    event: WindowEvent::RedrawRequested,
                    ..
                } => {
                    let image_extent: [u32; 2] = window.inner_size().into();

                    if image_extent.contains(&0) {
                        return;
                    }

                    previous_frame_end.as_mut().unwrap().cleanup_finished();

                    if recreate_swapchain {
                        let (new_swapchain, new_images) = swapchain
                            .recreate(SwapchainCreateInfo {
                                image_extent,
                                ..swapchain.create_info()
                            })
                            .expect("failed to recreate swapchain");

                        swapchain = new_swapchain;
                        framebuffers =
                            adjust_window_size(&new_images, render_pass.clone(), &mut viewport);
                        recreate_swapchain = false;
                    }

                    let (image_index, suboptimal, acquire_future) = match acquire_next_image(
                        swapchain.clone(),
                        None,
                    )
                    .map_err(Validated::unwrap)
                    {
                        Ok(r) => r,
                        Err(VulkanError::OutOfDate) => {
                            recreate_swapchain = true;
                            return;
                        }
                        Err(e) => panic!("failed to acquire next image: {e}"),
                    };

                    if suboptimal {
                        recreate_swapchain = true;
                    }

                    let mut builder = AutoCommandBufferBuilder::primary(
                        &command_buffer_allocator,
                        queue.queue_family_index(),
                        CommandBufferUsage::OneTimeSubmit,
                    )
                    .unwrap();

                    let upload_buffer: Subbuffer<[u8]> = Buffer::new_slice(
                        memory_allocator.clone(),
                        BufferCreateInfo {
                            usage: BufferUsage::TRANSFER_SRC,
                            ..Default::default()
                        },
                        AllocationCreateInfo {
                            memory_type_filter: MemoryTypeFilter::PREFER_HOST
                                | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                            ..Default::default()
                        },
                        //(info.width * info.height * 4) as DeviceSize,
                        (256 * 224 * 4) as DeviceSize,
                    )
                    .unwrap();

                    copy_screen_memory(&state, &upload_buffer);

                    builder
                        .begin_render_pass(
                            RenderPassBeginInfo {
                                clear_values: vec![Some([0.0, 0.0, 1.0, 1.0].into())],
                                ..RenderPassBeginInfo::framebuffer(
                                    framebuffers[image_index as usize].clone(),
                                )
                            },
                            Default::default(),
                        )
                        .unwrap()
                        .set_viewport(0, [viewport.clone()].into_iter().collect())
                        .unwrap()
                        .bind_pipeline_graphics(pipeline.clone())
                        .unwrap()
                        .bind_descriptor_sets(
                            PipelineBindPoint::Graphics,
                            pipeline.layout().clone(),
                            0,
                            set.clone(),
                        )
                        .unwrap()
                        .bind_vertex_buffers(0, vertex_buffer.clone())
                        .unwrap()
                        .bind_index_buffer(index_buffer.clone())
                        .unwrap()
                        .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
                        .unwrap()
                        .end_render_pass(Default::default())
                        .unwrap()
                        .copy_buffer_to_image(CopyBufferToImageInfo::buffer_image(
                            upload_buffer.clone(),
                            image.clone(),
                        ))
                        .unwrap();

                    let command_buffer = builder.build().unwrap();

                    let future = previous_frame_end
                        .take()
                        .unwrap()
                        .join(acquire_future)
                        .then_execute(queue.clone(), command_buffer)
                        .unwrap()
                        .then_swapchain_present(
                            queue.clone(),
                            SwapchainPresentInfo::swapchain_image_index(
                                swapchain.clone(),
                                image_index,
                            ),
                        )
                        .then_signal_fence_and_flush();

                    match future.map_err(Validated::unwrap) {
                        Ok(future) => {
                            previous_frame_end = Some(future.boxed());
                        }
                        Err(VulkanError::OutOfDate) => {
                            recreate_swapchain = true;
                            previous_frame_end = Some(sync::now(device.clone()).boxed());
                        }
                        Err(e) => {
                            println!("failed to flush future: {e}");
                            previous_frame_end = Some(sync::now(device.clone()).boxed());
                        }
                    }
                }
                Event::AboutToWait => window.request_redraw(),
                _ => (),
            }
        })
        .unwrap();

    handle.join().unwrap();
}
