use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints};
use std::fmt;

#[derive(Default, PartialEq)]
enum PlotType {
    #[default]
    Impulse,
    Step,
}

impl fmt::Display for PlotType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Impulse => write!(f, "Impulse"),
            Self::Step => write!(f, "Step"),
        }
    }
}

#[derive(Default, PartialEq)]
enum WindowType {
    #[default]
    Rectangular,
    Triangular,
    Welch,
    Sine,
    Hann,
    Hamming,
    Blackman,
    Nuttall,
    BlackmanNuttall,
    BlackmanHarris,
    FlatTop,
}

impl fmt::Display for WindowType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rectangular => write!(f, "Rectangular"),
            Self::Triangular => write!(f, "Triangular"),
            Self::Welch => write!(f, "Welch"),
            Self::Sine => write!(f, "Sine"),
            Self::Hann => write!(f, "Hann"),
            Self::Hamming => write!(f, "Hamming"),
            Self::Blackman => write!(f, "Blackman"),
            Self::Nuttall => write!(f, "Nutall"),
            Self::BlackmanNuttall => write!(f, "Blackman Nutall"),
            Self::BlackmanHarris => write!(f, "Blackman Harris"),
            Self::FlatTop => write!(f, "Flat Top"),
        }
    }
}

#[derive(Default, PartialEq)]
enum FilterType {
    #[default]
    LowPass,
    HighPass,
    BandPass,
    BandStop,
}

impl fmt::Display for FilterType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LowPass => write!(f, "Low Pass"),
            Self::HighPass => write!(f, "High Pass"),
            Self::BandPass => write!(f, "Band Pass"),
            Self::BandStop => write!(f, "Band Stop"),
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions::default();

    eframe::run_native(
        "FIR Filter Designer",
        options,
        Box::new(|cc| Box::new(App::new(cc))),
    )
}

#[derive(Default)]
struct App {
    sampling_freq: usize,
    filter_len: usize,
    filter_shift: usize,
    filter_type: FilterType,
    low_freq: usize,
    high_freq: usize,
    window_type: WindowType,
    plot_type: PlotType,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut plot_rect_filter_resp_time = None;
        let mut plot_rect_filter_resp_freq = None;
        let mut plot_rect_window_resp_time = None;
        let mut plot_rect_window_resp_freq = None;

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.label("Filter Parameters");
            ui.separator();
            egui::Grid::new("filter_params").show(ui, |ui| {
                ui.label("Sampling Frequency (Hz):");
                ui.add(
                    egui::DragValue::new(&mut self.sampling_freq)
                        .speed(0.1)
                        .max_decimals(0),
                );
                ui.end_row();

                ui.label("Filter Length (Samples):");
                ui.add(
                    egui::DragValue::new(&mut self.filter_len)
                        .speed(0.1)
                        .max_decimals(0),
                );
                ui.end_row();

                ui.label("Filter Shift (Samples):");
                ui.add(
                    egui::DragValue::new(&mut self.filter_shift)
                        .speed(0.1)
                        .max_decimals(0),
                );
                ui.end_row();

                ui.label("Filter Type:");
                egui::ComboBox::from_id_source("filter_type")
                    .selected_text(format!("{}", self.filter_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.filter_type, FilterType::LowPass, "Low Pass");
                        ui.selectable_value(
                            &mut self.filter_type,
                            FilterType::HighPass,
                            "High Pass",
                        );
                        ui.selectable_value(
                            &mut self.filter_type,
                            FilterType::BandPass,
                            "Band Pass",
                        );
                        ui.selectable_value(
                            &mut self.filter_type,
                            FilterType::BandStop,
                            "Band Stop",
                        );
                    });
                ui.end_row();

                match self.filter_type {
                    FilterType::LowPass => {
                        ui.label("High Cut Frequency (Hz):");
                        ui.add(
                            egui::DragValue::new(&mut self.high_freq)
                                .speed(0.1)
                                .max_decimals(0),
                        );
                    }
                    _ => {
                        ui.label("Low Cut Frequency (Hz):");
                        ui.add(
                            egui::DragValue::new(&mut self.low_freq)
                                .speed(0.1)
                                .max_decimals(0),
                        );
                    }
                };
                ui.end_row();

                match self.filter_type {
                    FilterType::HighPass | FilterType::LowPass => (),
                    _ => {
                        ui.label("High Cut Frequency (Hz):");
                        ui.add(
                            egui::DragValue::new(&mut self.high_freq)
                                .speed(0.1)
                                .max_decimals(0),
                        );
                        ui.end_row();
                    }
                };

                ui.label("Window Type:");
                egui::ComboBox::from_id_source("window_type")
                    .selected_text(format!("{}", self.window_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.window_type,
                            WindowType::Rectangular,
                            "Rectangular",
                        );
                        ui.selectable_value(
                            &mut self.window_type,
                            WindowType::Triangular,
                            "Triangular",
                        );
                        ui.selectable_value(&mut self.window_type, WindowType::Welch, "Welch");
                        ui.selectable_value(&mut self.window_type, WindowType::Sine, "Sine");
                        ui.selectable_value(&mut self.window_type, WindowType::Hann, "Hann");
                        ui.selectable_value(&mut self.window_type, WindowType::Hamming, "Hamming");
                        ui.selectable_value(
                            &mut self.window_type,
                            WindowType::Blackman,
                            "Blackman",
                        );
                        ui.selectable_value(&mut self.window_type, WindowType::Nuttall, "Nutall");
                        ui.selectable_value(
                            &mut self.window_type,
                            WindowType::BlackmanNuttall,
                            "Blackman Nuttull",
                        );
                        ui.selectable_value(
                            &mut self.window_type,
                            WindowType::BlackmanHarris,
                            "Blackman Harris",
                        );
                        ui.selectable_value(&mut self.window_type, WindowType::FlatTop, "Flat Top");
                    });
            });

            ui.add_space(40.0);
            ui.label("Plot");
            ui.separator();
            egui::Grid::new("plot_params").show(ui, |ui| {
                ui.label("Plot Response:");
                egui::ComboBox::from_id_source("plot_type")
                    .selected_text(format!("{}", self.plot_type))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.plot_type, PlotType::Impulse, "Impulse");
                        ui.selectable_value(&mut self.plot_type, PlotType::Step, "Step");
                    });
            });
            ui.add_space(40.0);
            ui.label("File");
            ui.separator();
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    if ui.button("Export Filter").clicked() {}

                    if ui.button("Save Plots").clicked() {};
                },
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let plot_width = (ui.max_rect().width() - 4.0 * ui.spacing().item_spacing.x) / 2.0;
            let plot_height = (ui.max_rect().height() - 20.0 * ui.spacing().item_spacing.y) / 2.0;

            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.scope(|ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.label("Filter Response (Time Domain)");
                        let plot_filter_resp_time = Plot::new("filter_resp_time")
                            .width(plot_width)
                            .height(plot_height)
                            .allow_scroll(false)
                            .x_axis_label("Time (s)")
                            .y_axis_width(1);

                        // let's create a dummy line in the plot
                        let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0]];
                        let inner = plot_filter_resp_time.show(ui, |plot_ui| {
                            plot_ui.line(Line::new(PlotPoints::from(graph)).name("curve"));
                        });

                        // Remember the position of the plot
                        plot_rect_filter_resp_time = Some(inner.response.rect);

                        ui.add_space(10.0);
                        ui.label("Filter Response (Frequency Domain)");
                        let plot_filter_resp_freq = Plot::new("filter_resp_freq")
                            .width(plot_width)
                            .height(plot_height)
                            .allow_scroll(false)
                            .x_axis_label("Frequency (Hz)")
                            .y_axis_width(1);

                        // let's create a dummy line in the plot
                        let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0]];
                        let inner = plot_filter_resp_freq.show(ui, |plot_ui| {
                            plot_ui.line(Line::new(PlotPoints::from(graph)).name("curve"));
                        });

                        // Remember the position of the plot
                        plot_rect_filter_resp_freq = Some(inner.response.rect);
                    });
                });

                ui.separator();
                ui.scope(|ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.label("Window Function (Time Domain)");
                        let plot_window_resp_time = Plot::new("window_resp_time")
                            .width(plot_width)
                            .height(plot_height)
                            .allow_scroll(false)
                            .x_axis_label("Time (s)")
                            .y_axis_width(1);

                        // let's create a dummy line in the plot
                        let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0]];
                        let inner = plot_window_resp_time.show(ui, |plot_ui| {
                            plot_ui.line(Line::new(PlotPoints::from(graph)).name("curve"));
                        });

                        // Remember the position of the plot
                        plot_rect_window_resp_time = Some(inner.response.rect);

                        ui.add_space(10.0);
                        ui.label("Window Function (Frequency Domain)");
                        let plot_window_resp_freq = Plot::new("window_resp_freq")
                            .width(plot_width)
                            .height(plot_height)
                            .allow_scroll(false)
                            .x_axis_label("Frequency (Hz)")
                            .y_axis_width(1);

                        // let's create a dummy line in the plot
                        let graph: Vec<[f64; 2]> = vec![[0.0, 1.0], [2.0, 3.0], [3.0, 2.0]];
                        let inner = plot_window_resp_freq.show(ui, |plot_ui| {
                            plot_ui.line(Line::new(PlotPoints::from(graph)).name("curve"));
                        });

                        // Remember the position of the plot
                        plot_rect_window_resp_freq = Some(inner.response.rect);
                    });
                });
            });
        });
    }
}
