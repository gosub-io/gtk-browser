use gtk4::prelude::*;
use gtk4::subclass::prelude::*;
use std::cell::RefCell;
use gtk4::glib::ControlFlow::Continue;
use vello::wgpu::TextureFormat;
use crate::runtime;

mod imp {
    use super::*;
    use gtk4::glib;

    #[derive(Default)]
    pub struct VelloWidget {
        pub state: RefCell<Option<super::VelloState>>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for VelloWidget {
        const NAME: &'static str = "VelloWidget";
        type Type = super::VelloWidget;
        type ParentType = gtk4::Widget;
    }

    impl ObjectImpl for VelloWidget {
        fn constructed(&self) {
            self.parent_constructed();

            self.set_has_window(false);
        }
    }

    impl WidgetImpl for VelloWidget {
        fn realize(&self) {
            self.parent_realize();
            super::initialize_renderer(&self, &self.state);
        }

        fn size_allocate(&self, width: i32, height: i32, baseline: i32) {
            self.parent_size_allocate(width, height, baseline);

            if let Some(state) = &*self.state.borrow() {
                state.resize(width, height);
            }
        }
    }
}

glib::wrapper! {
    pub struct VelloWidget(ObjectSubclass<imp::VelloWidget>)
        @extends gtk4::Widget;
}

impl VelloWidget {
    pub fn new() -> Self {
        glib::Object::new().expect("Failed to create VelloWidget")
    }

    pub fn initialize_renderer(widget: &Self, state_cell: &RefCell<Option<VelloState>>) {
        use gtk4::prelude::*;
        use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};

        let gdk_surface = widget.native().unwrap().surface().unwrap();

        let raw_window_handle = gdk_surface.raw_window_handle();
        // let raw_display_handle = gdk_surface.raw_display_handle();

        let instance = wgpu::Instance::new(wgpu::Backends::PRIMARY);

        let surface = unsafe { instance.create_surface((&gdk_surface).expect("Failed to create surface")) };

        let adapter = runtime().block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })).expect("Failed to request adapter");

        let (device, queue) = runtime().block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            features: wgpu::Features::empty(),
            limits: wgpu::Limits::default(),
            label: None,
        }, None)).expect("Failed to request device");

        let renderer = vello::Renderer::new(&device).expect("Failed to create renderer");

        let state = VelloState {
            surface,
            device,
            queue,
            renderer,
            bounds: widget.compute_bounds(&gdk_surface),
        };

        state_cell.replace(Some(state));
    }
}

struct VelloState {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: vello::Renderer,
    bounds: Option<gtk4::graphene::Rect>,
}

impl VelloState {
    fn resize(&self, width: i32, height: i32) {
        let config = wgpu::SurfaceConfiguration {
            usage: self.surface.get_capabilities(&self.device.adapter()).formats[0],
            format: TextureFormat::R8Unorm,
            width: width as u32,
            height: height as u32,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            view_formats: vec![],
        };

        self.surface.configure(&self.device, &config);
    }
}

impl VelloWidget {
    fn start_rendering(&self) {
        // let widget_clone = self.clone();
        self.add_tick_callback(move |_widget, _clock | {
            Continue
        });
    }
}