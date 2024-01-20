mod disassemble;
mod emulate8080;
mod i8080;
mod shaders;

use std::env;
use std::fs;
use std::io;
use std::sync::{Arc, Mutex};
use std::thread;

use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage};
use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::command_buffer::RenderingAttachmentInfo;
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, RenderingInfo};
use vulkano::device::physical::{PhysicalDevice, PhysicalDeviceType};
use vulkano::device::DeviceExtensions;
use vulkano::device::Features;
use vulkano::device::{Device, DeviceCreateInfo, QueueCreateInfo, QueueFlags};
use vulkano::image::view::ImageView;
use vulkano::image::{Image, ImageUsage};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::subpass::PipelineRenderingCreateInfo;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::DynamicState;
use vulkano::pipeline::{GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::AttachmentLoadOp;
use vulkano::swapchain::{self, Surface, Swapchain, SwapchainCreateInfo, SwapchainPresentInfo};
use vulkano::sync::{self, GpuFuture};
use vulkano::Validated;
use vulkano::Version;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;

use crate::emulate8080::run_emulation;

#[derive(BufferContents, Vertex)]
#[repr(C)]
pub struct Vertex2D {
    #[format(R32G32_SFLOAT)]
    position: [f32; 2],

    #[name("color")]
    #[format(R32G32B32_SFLOAT)]
    color: [f32; 3],
}

fn adjust_window_size(images: &[Arc<Image>], viewport: &mut Viewport) -> Vec<Arc<ImageView>> {
    let extent = images[0].extent();
    viewport.extent = [extent[0] as f32, extent[1] as f32];

    images
        .iter()
        .map(|image| ImageView::new_default(image.clone()).unwrap())
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
        .filter(|pd| {
            (pd.api_version() >= Version::V1_3 || pd.supported_extensions().khr_dynamic_rendering)
                && pd.supported_extensions().contains(&device_extensions)
        })
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

    let library = vulkano::VulkanLibrary::new().expect("No local Vulkan library/DLL");
    let required_extensions = Surface::required_extensions(&event_loop);

    let instance = Instance::new(
        library,
        InstanceCreateInfo {
            enabled_extensions: required_extensions,
            ..Default::default()
        },
    )
    .expect("Failed to create instance");

    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

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
            enabled_features: Features {
                dynamic_rendering: true,
                ..Features::empty()
            },
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
            surface.clone(),
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

        let subpass = PipelineRenderingCreateInfo {
            color_attachment_formats: vec![Some(swapchain.image_format())],
            ..Default::default()
        };

        GraphicsPipeline::new(
            device.clone(),
            None,
            GraphicsPipelineCreateInfo {
                stages: stages.into_iter().collect(),
                vertex_input_state: Some(vertex_input_state),
                input_assembly_state: Some(InputAssemblyState::default()),
                viewport_state: Some(ViewportState::default()),
                rasterization_state: Some(RasterizationState::default()),
                multisample_state: Some(MultisampleState::default()),
                color_blend_state: Some(ColorBlendState::with_attachment_states(
                    subpass.color_attachment_formats.len() as u32,
                    ColorBlendAttachmentState::default(),
                )),
                dynamic_state: [DynamicState::Viewport].into_iter().collect(),
                subpass: Some(subpass.into()),
                ..GraphicsPipelineCreateInfo::layout(layout)
            },
        )
        .unwrap()
    };

    let mut viewport = Viewport {
        offset: [0.0, 0.0],
        extent: [0.0, 0.0],
        depth_range: 0.0..=1.0,
    };

    let mut attachment_image_views = adjust_window_size(&images, &mut viewport);

    let command_buffer_allocator =
        StandardCommandBufferAllocator::new(device.clone(), Default::default());

    let vertex1 = Vertex2D {
        position: [-0.5, -0.5],
        color: [1.0, 0.0, 0.0],
    };
    let vertex2 = Vertex2D {
        position: [-0.5, 0.5],
        color: [0.0, 1.0, 0.0],
    };
    let vertex3 = Vertex2D {
        position: [0.5, -0.5],
        color: [0.0, 0.0, 1.0],
    };
    let vertex4 = Vertex2D {
        position: [0.5, 0.5],
        color: [1.0, 0.0, 1.0],
    };

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
        vec![vertex1, vertex2, vertex3, vertex4],
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

    let mut recreate_swapchain = false;

    let mut previous_fence = Some(sync::now(device.clone()).boxed());

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
                Event::AboutToWait => {
                    let image_extent: [u32; 2] = window.inner_size().into();

                    if image_extent.contains(&0) {
                        return;
                    }

                    previous_fence.as_mut().unwrap().cleanup_finished();

                    if recreate_swapchain {
                        recreate_swapchain = false;

                        let (new_swapchain, new_images) = swapchain
                            .recreate(SwapchainCreateInfo {
                                image_extent,
                                ..swapchain.create_info()
                            })
                            .expect("Failed to recreate swapchain");

                        swapchain = new_swapchain;

                        attachment_image_views = adjust_window_size(&new_images, &mut viewport);
                    }

                    let (image_index, suboptimal, acquire_future) =
                        match swapchain::acquire_next_image(swapchain.clone(), None)
                            .map_err(Validated::unwrap)
                        {
                            Ok(r) => r,
                            Err(vulkano::VulkanError::OutOfDate) => {
                                recreate_swapchain = true;
                                return;
                            }
                            Err(e) => panic!("Failed to acquire next image: {e}"),
                        };

                    if suboptimal {
                        recreate_swapchain = true;
                    }

                    //let mut builder = AutoCommandBufferBuilder::
                    let mut builder = AutoCommandBufferBuilder::primary(
                        &command_buffer_allocator,
                        queue.queue_family_index(),
                        CommandBufferUsage::MultipleSubmit,
                    )
                    .unwrap();

                    builder
                        .begin_rendering(RenderingInfo {
                            color_attachments: vec![Some(RenderingAttachmentInfo {
                                load_op: AttachmentLoadOp::Clear,
                                store_op: vulkano::render_pass::AttachmentStoreOp::Store,
                                clear_value: Some([0.0, 0.0, 1.0, 1.0].into()),
                                ..RenderingAttachmentInfo::image_view(
                                    attachment_image_views[image_index as usize].clone(),
                                )
                            })],
                            ..Default::default()
                        })
                        .unwrap()
                        .set_viewport(0, [viewport.clone()].into_iter().collect())
                        .unwrap()
                        .bind_pipeline_graphics(pipeline.clone())
                        .unwrap()
                        .bind_vertex_buffers(0, vertex_buffer.clone())
                        .unwrap()
                        .bind_index_buffer(index_buffer.clone())
                        .unwrap()
                        .draw_indexed(index_buffer.len() as u32, 1, 0, 0, 0)
                        .unwrap()
                        .end_rendering()
                        .unwrap();

                    let command_buffer = builder.build().unwrap();

                    let future = previous_fence
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
                            previous_fence = Some(future.boxed());
                        }
                        Err(vulkano::VulkanError::OutOfDate) => {
                            recreate_swapchain = true;
                            previous_fence = Some(sync::now(device.clone()).boxed());
                        }
                        Err(e) => panic!("Failed to flush future: {e}"),
                    }
                }
                _ => (),
            }
        })
        .unwrap();

    handle.join().unwrap();
}
