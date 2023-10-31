use super::fir::{Filter, FilterDef, Window};
use eframe::egui;
use egui_plot::{Line, Plot};
use std::f64::consts::PI;
use std::fmt;

const DFT_LEN: usize = 256;

pub struct FilterData {
    filter: Vec<f64>,
    window: Vec<f64>,
    f_windowed: Vec<f64>,
    filter_imp: Vec<[f64; 2]>,
    filter_stp: Vec<[f64; 2]>,
    filter_dft: Vec<[f64; 2]>,
    window_fun: Vec<[f64; 2]>,
    window_dft: Vec<[f64; 2]>,
    f_windowed_imp: Vec<[f64; 2]>,
    f_windowed_stp: Vec<[f64; 2]>,
    f_windowed_dft: Vec<[f64; 2]>,
}

impl From<&FilterDef> for FilterData {
    fn from(def: &FilterDef) -> Self {
        let filter = def.compute_filter();
        let window = def.compute_window();
        let f_windowed = FilterDef::compute_filter_windowed(&filter, &window);
        let filter_imp = plot_filter_imp(&filter, def.f_sampling);
        let filter_stp = plot_filter_stp(&filter, def.f_sampling);
        let filter_dft = plot_dft(&filter, def.f_sampling);
        let window_fun = plot_window(&window, def.f_sampling);
        let window_dft = plot_dft(&window, def.f_sampling);
        let f_windowed_imp = plot_filter_imp(&f_windowed, def.f_sampling);
        let f_windowed_stp = plot_filter_stp(&f_windowed, def.f_sampling);
        let f_windowed_dft = plot_dft(&f_windowed, def.f_sampling);

        Self {
            filter,
            window,
            f_windowed,
            filter_imp,
            filter_stp,
            filter_dft,
            window_fun,
            window_dft,
            f_windowed_imp,
            f_windowed_stp,
            f_windowed_dft,
        }
    }
}

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

pub struct App {
    filter_def: FilterDef,
    filter_data: FilterData,
    plot_type: PlotType,
    show_window: bool,
}

impl App {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut filter_def = FilterDef::default();
        filter_def.f_sampling = 1000.0;
        filter_def.len = 64;
        filter_def.shift = 32;
        filter_def.f_lo_cut = 100.0;
        filter_def.f_hi_cut = 300.0;

        let filter_data = FilterData::from(&filter_def);

        Self {
            filter_def,
            filter_data,
            plot_type: PlotType::default(),
            show_window: true,
        }
    }

    fn draw_window_combo_box(&mut self, ui: &mut egui::Ui) {
        egui::ComboBox::from_id_source("window_type")
            .selected_text(format!("{}", self.filter_def.window))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.filter_def.window,
                    Window::Rectangular,
                    "Rectangular",
                );
                ui.selectable_value(
                    &mut self.filter_def.window,
                    Window::Triangular,
                    "Triangular",
                );
                ui.selectable_value(&mut self.filter_def.window, Window::Welch, "Welch");
                ui.selectable_value(&mut self.filter_def.window, Window::Sin, "Sin");
                ui.selectable_value(&mut self.filter_def.window, Window::Hann, "Hann");
                ui.selectable_value(&mut self.filter_def.window, Window::Hamming, "Hamming");
                ui.selectable_value(&mut self.filter_def.window, Window::Blackman, "Blackman");
                ui.selectable_value(&mut self.filter_def.window, Window::Nuttall, "Nutall");
                ui.selectable_value(
                    &mut self.filter_def.window,
                    Window::BlackmanNuttall,
                    "Blackman Nuttull",
                );
                ui.selectable_value(
                    &mut self.filter_def.window,
                    Window::BlackmanHarris,
                    "Blackman Harris",
                );
                ui.selectable_value(&mut self.filter_def.window, Window::FlatTop, "Flat Top");
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let filter_def_prev = self.filter_def.clone();

        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.add_space(4.0);
            ui.label("Filter Parameters");
            ui.separator();
            egui::Grid::new("filter").show(ui, |ui| {
                ui.label("Sampling Frequency (Hz):");
                ui.add(
                    egui::DragValue::new(&mut self.filter_def.f_sampling)
                        .speed(0.1)
                        .clamp_range(0.0..=f64::NAN)
                        .max_decimals(0),
                );
                ui.end_row();

                ui.label("Filter Length (Samples):");
                ui.add(
                    egui::DragValue::new(&mut self.filter_def.len)
                        .speed(0.1)
                        .max_decimals(0),
                );
                ui.end_row();

                ui.label("Filter Shift (Samples):");
                ui.add(
                    egui::DragValue::new(&mut self.filter_def.shift)
                        .speed(0.1)
                        .max_decimals(0),
                );
                ui.end_row();

                ui.label("Filter Type:");
                egui::ComboBox::from_id_source("filter")
                    .selected_text(format!("{}", self.filter_def.filter))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.filter_def.filter,
                            Filter::LowPass,
                            "Low Pass",
                        );
                        ui.selectable_value(
                            &mut self.filter_def.filter,
                            Filter::HighPass,
                            "High Pass",
                        );
                        ui.selectable_value(
                            &mut self.filter_def.filter,
                            Filter::BandPass,
                            "Band Pass",
                        );
                        ui.selectable_value(
                            &mut self.filter_def.filter,
                            Filter::BandStop,
                            "Band Stop",
                        );
                    });
                ui.end_row();

                match self.filter_def.filter {
                    Filter::LowPass => {
                        ui.label("High Cut Frequency (Hz):");
                        ui.add(
                            egui::DragValue::new(&mut self.filter_def.f_hi_cut)
                                .speed(0.1)
                                .clamp_range(0.0..=f64::NAN)
                                .max_decimals(0),
                        );
                    }
                    _ => {
                        ui.label("Low Cut Frequency (Hz):");
                        ui.add(
                            egui::DragValue::new(&mut self.filter_def.f_lo_cut)
                                .speed(0.1)
                                .clamp_range(0.0..=f64::NAN)
                                .max_decimals(0),
                        );
                    }
                };
                ui.end_row();

                match self.filter_def.filter {
                    Filter::HighPass | Filter::LowPass => (),
                    _ => {
                        ui.label("High Cut Frequency (Hz):");
                        ui.add(
                            egui::DragValue::new(&mut self.filter_def.f_hi_cut)
                                .speed(0.1)
                                .clamp_range(0.0..=f64::NAN)
                                .max_decimals(0),
                        );
                        ui.end_row();
                    }
                };

                ui.label("Window Type:");
                self.draw_window_combo_box(ui);
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
                ui.end_row();

                ui.label("Show Window:");
                ui.checkbox(&mut self.show_window, "");
            });

            ui.add_space(40.0);
            ui.label("File");
            ui.separator();
            ui.with_layout(
                egui::Layout::top_down_justified(egui::Align::Center),
                |ui| {
                    if ui.button("Export Filter").clicked() {
                        println! {"{:?}", self.filter_data.f_windowed};
                    }

                    if ui.button("Save Plots").clicked() {};
                },
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            let mut plot_width = ui.max_rect().width() - 4.0 * ui.spacing().item_spacing.x;
            let plot_height = (ui.max_rect().height() - 20.0 * ui.spacing().item_spacing.y) / 2.0;

            if self.show_window {
                plot_width *= 0.5;
            }

            ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                ui.scope(|ui| {
                    ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                        ui.label("Filter Response (Time Domain)");
                        let plot_filter_resp_time = Plot::new("filter_resp_time")
                            .width(plot_width)
                            .height(plot_height)
                            .allow_scroll(false)
                            .x_axis_label("Time (s)")
                            .y_axis_width(3)
                            .legend(
                                egui_plot::Legend::default().text_style(egui::TextStyle::Small),
                            );

                        plot_filter_resp_time.show(ui, |plot_ui| match self.plot_type {
                            PlotType::Impulse => {
                                plot_ui.line(
                                    Line::new(self.filter_data.filter_imp.clone()).name("Filter"),
                                );
                                plot_ui.line(
                                    Line::new(self.filter_data.f_windowed_imp.clone())
                                        .name("Windowed"),
                                );
                            }
                            PlotType::Step => {
                                plot_ui.line(
                                    Line::new(self.filter_data.filter_stp.clone()).name("Filter"),
                                );
                                plot_ui.line(
                                    Line::new(self.filter_data.f_windowed_stp.clone())
                                        .name("Windowed"),
                                );
                            }
                        });

                        ui.add_space(10.0);
                        ui.label("Filter Response (Frequency Domain)");
                        let plot_filter_resp_freq = Plot::new("filter_resp_freq")
                            .width(plot_width)
                            .height(plot_height)
                            .allow_scroll(false)
                            .x_axis_label("Frequency (Hz)")
                            .y_axis_width(3)
                            .legend(
                                egui_plot::Legend::default().text_style(egui::TextStyle::Small),
                            );

                        plot_filter_resp_freq.show(ui, |plot_ui| {
                            plot_ui.line(
                                Line::new(self.filter_data.filter_dft.clone()).name("Filter"),
                            );
                            plot_ui.line(
                                Line::new(self.filter_data.f_windowed_dft.clone()).name("Windowed"),
                            );
                        });
                    });
                });

                if self.show_window {
                    ui.separator();
                    ui.scope(|ui| {
                        ui.with_layout(egui::Layout::top_down(egui::Align::LEFT), |ui| {
                            ui.label("Window Function (Time Domain)");
                            let plot_window_resp_time = Plot::new("window_resp_time")
                                .width(plot_width)
                                .height(plot_height)
                                .allow_scroll(false)
                                .x_axis_label("Time (s)")
                                .y_axis_width(3);

                            plot_window_resp_time.show(ui, |plot_ui| {
                                plot_ui.line(Line::new(self.filter_data.window_fun.clone()));
                            });

                            ui.add_space(10.0);
                            ui.label("Window Function (Frequency Domain)");
                            let plot_window_resp_freq = Plot::new("window_resp_freq")
                                .width(plot_width)
                                .height(plot_height)
                                .allow_scroll(false)
                                .x_axis_label("Frequency (Hz)")
                                .y_axis_width(3);

                            plot_window_resp_freq.show(ui, |plot_ui| {
                                plot_ui.line(Line::new(self.filter_data.window_dft.clone()));
                            });
                        });
                    });
                }
            });
        });

        if filter_def_prev != self.filter_def {
            self.filter_data = FilterData::from(&self.filter_def);
        }
    }
}

fn plot_window(w: &Vec<f64>, f_sampling: f64) -> Vec<[f64; 2]> {
    let dt = 1.0 / f_sampling;

    let mut n = 0;
    w.into_iter()
        .map(|w| {
            let t = n as f64 * dt;
            n += 1;

            [t, *w]
        })
        .collect()
}

fn plot_filter_imp(f: &Vec<f64>, f_sampling: f64) -> Vec<[f64; 2]> {
    let dt = 1.0 / f_sampling;

    let mut n = 0;
    f.into_iter()
        .map(|f| {
            let t = n as f64 * dt;
            n += 1;

            [t, f * dt]
        })
        .collect()
}

fn plot_filter_stp(f: &Vec<f64>, f_sampling: f64) -> Vec<[f64; 2]> {
    let dt = 1.0 / f_sampling;

    let mut n = 0;
    let mut f_prev = 0.0;
    let mut y = 0.0;
    f.into_iter()
        .map(|f| {
            let t = n as f64 * dt;
            n += 1;

            y += f_prev + dt * 0.5 * (f + f_prev);
            f_prev = *f;

            [t, y]
        })
        .collect()
}

/// Returns the amplitude of the DFT of a signal.
///
/// The index `$m$` runs from 0 to `$\frac{N}{2}$`. This automatically discards the negative frequency
/// components produced by the DFT. Considering that for this use case, the filter length will be less
/// than `$N$`, the index `$n$` running over the filter length effectly results in a zero padded signal.
///
/// Instead of using the complex function:
///
/// ```math
/// e^{-j2\pi nm/N}
/// ```
/// Eulers formula is used:
/// ```math
/// c_m[n] = \cos(2\pi mn/N) \\
/// s_m[n] = \sin(2\pi mn/N)
/// ```
///
/// [\[1\]](https://hal.science/hal-04075823/document) Laurent Nony, Jean-Marc Themlin.
/// An introduction to the Discrete Fourier Transform and its applications in signal processing. Master. France. 2023. hal-04075823
///
/// [\[2\]](http://www.dspguide.com/pdfbook.htm) Steven W. Smith.
/// The Scientist and Engineer's Guide to Digital Signal Processing
fn plot_dft(signal: &Vec<f64>, f_sampling: f64) -> Vec<[f64; 2]> {
    let df = f_sampling / (DFT_LEN) as f64;
    let f: Vec<f64> = (0..DFT_LEN / 2).map(|n| n as f64 * df).collect();

    let dft: Vec<f64> = (0..DFT_LEN / 2)
        .map(|m| {
            let mut n = 0;
            let (re, im) = signal.into_iter().fold((0.0, 0.0), |(mut re, mut im), x| {
                let theta = 2.0 * PI * (m * n) as f64 / DFT_LEN as f64;

                n += 1;
                re += x * (theta).cos();
                im -= x * (theta).sin();

                (re, im)
            });

            // Calculate the DFT magnitude in dB
            20.0 * ((re.powi(2) + im.powi(2)).sqrt()).log10()
        })
        .collect();

    let plot: Vec<[f64; 2]> = f.iter().zip(dft).map(|(f, y)| [*f, y]).collect();
    plot
}
