use std::f64::consts::PI;
use std::fmt;

#[derive(Default, PartialEq, Clone)]
pub struct FilterDef {
    pub filter: Filter,
    pub window: Window,
    pub len: usize,
    pub shift: usize,
    pub f_sampling: f64,
    pub f_lo_cut: f64,
    pub f_hi_cut: f64,
}

impl FilterDef {
    pub fn compute_filter(&self) -> Vec<f64> {
        let filter_fn = self.filter.function();
        let dt = 1.0 / self.f_sampling as f64;

        let f = (0..self.len)
            .map(|n| {
                filter_fn(
                    n,
                    self.shift,
                    dt,
                    self.f_lo_cut as f64,
                    self.f_hi_cut as f64,
                )
            })
            .collect();
        let g = match self.filter {
            Filter::LowPass | Filter::BandStop => FilterDef::compute_dc_gain(&f),
            Filter::BandPass => FilterDef::compute_gain(
                &f,
                self.f_lo_cut + (self.f_hi_cut - self.f_lo_cut) / 2 as f64,
            ),
            Filter::HighPass => FilterDef::compute_gain(&f, self.f_sampling / 2 as f64),
        };
        FilterDef::normalize_filter(&f, g)
    }

    pub fn compute_window(&self) -> Vec<f64> {
        let window_fn = self.window.function();

        (0..self.len).map(|n| window_fn(n, self.len - 1)).collect()
    }

    pub fn compute_filter_windowed(f: &Vec<f64>, w: &Vec<f64>) -> Vec<f64> {
        if f.len() != w.len() {
            panic!("fn_compute_filter_windowed: cannot multiply vec of different lengths")
        }

        f.iter().zip(w).map(|(f, w)| f * w).collect()
    }

    pub fn compute_dc_gain(f: &Vec<f64>) -> f64 {
        f.into_iter().fold(0.0, |mut g, h| {
            g += h;
            g
        })
    }

    pub fn compute_gain(f: &Vec<f64>, w: f64) -> f64 {
        let mut n = 0;
        let (re, im) = f.into_iter().fold((0.0, 0.0), |(mut re, mut im), h| {
            re += h * (w * n as f64).cos();
            im -= h * (w * n as f64).sin();
            n += 1;

            (re, im)
        });

        (re.powi(2) + im.powi(2)).sqrt()
    }

    pub fn normalize_filter(f: &Vec<f64>, g: f64) -> Vec<f64> {
        f.into_iter().map(|h| h / g).collect()
    }
}

type WindowFn = fn(usize, usize) -> f64;

fn window_rectangular(_n: usize, _len: usize) -> f64 {
    1.0
}

fn window_triangular(n: usize, len: usize) -> f64 {
    let n = n as f64;
    let len = len as f64;

    1.0 - ((n - 0.5 * len) / (0.5 * len)).abs()
}

fn window_welch(n: usize, len: usize) -> f64 {
    let n = n as f64;
    let len = len as f64;

    1.0 - ((n - 0.5 * len) / (0.5 * len)).powi(2)
}

fn window_sin(n: usize, len: usize) -> f64 {
    let n = n as f64;
    let len = len as f64;

    (PI * n / len).sin()
}

fn window_hann(n: usize, len: usize) -> f64 {
    let n = n as f64;
    let len = len as f64;

    0.5 * (1.0 - (2.0 * PI * n / len).cos())
}

fn window_hamming(n: usize, len: usize) -> f64 {
    let n = n as f64;
    let len = len as f64;

    (25.0 / 46.0) - (21.0 / 46.0) * (2.0 * PI * n / len).cos()
}

fn window_blackman(n: usize, len: usize) -> f64 {
    let n = n as f64;
    let len = len as f64;

    0.42 - 0.5 * (2.0 * PI * n / len).cos() + 0.08 * (4.0 * PI * n / len).cos()
}

fn window_nuttall(n: usize, len: usize) -> f64 {
    let n = n as f64;
    let len = len as f64;

    0.355768 - 0.487396 * (2.0 * PI * n / len).cos() + 0.144232 * (4.0 * PI * n / len).cos()
        - 0.012604 * (6.0 * PI * n / len).cos()
}

fn window_blackman_nuttall(n: usize, len: usize) -> f64 {
    let n = n as f64;
    let len = len as f64;

    0.3635819 - 0.4891775 * (2.0 * PI * n / len).cos() + 0.1365995 * (4.0 * PI * n / len).cos()
        - 0.0106411 * (6.0 * PI * n / len).cos()
}

fn window_blackman_harris(n: usize, len: usize) -> f64 {
    let n = n as f64;
    let len = len as f64;

    0.35875 - 0.48829 * (2.0 * PI * n / len).cos() + 0.14128 * (4.0 * PI * n / len).cos()
        - 0.01168 * (6.0 * PI * n / len).cos()
}

fn window_flat_top(n: usize, len: usize) -> f64 {
    let n = n as f64;
    let len = len as f64;

    0.21557895 - 0.41663158 * (2.0 * PI * n / len).cos() + 0.277263158 * (4.0 * PI * n / len).cos()
        - 0.083578947 * (6.0 * PI * n / len).cos()
        + 0.006947368 * (8.0 * PI * n / len).cos()
}

#[derive(Default, PartialEq, Clone)]
pub enum Window {
    #[default]
    Rectangular,
    Triangular,
    Welch,
    Sin,
    Hann,
    Hamming,
    Blackman,
    Nuttall,
    BlackmanNuttall,
    BlackmanHarris,
    FlatTop,
}

impl Window {
    fn function(&self) -> WindowFn {
        match self {
            Self::Rectangular => window_rectangular,
            Self::Triangular => window_triangular,
            Self::Welch => window_welch,
            Self::Sin => window_sin,
            Self::Hann => window_hann,
            Self::Hamming => window_hamming,
            Self::Blackman => window_blackman,
            Self::Nuttall => window_nuttall,
            Self::BlackmanNuttall => window_blackman_nuttall,
            Self::BlackmanHarris => window_blackman_harris,
            Self::FlatTop => window_flat_top,
        }
    }
}

impl fmt::Display for Window {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rectangular => write!(f, "Rectangular"),
            Self::Triangular => write!(f, "Triangular"),
            Self::Welch => write!(f, "Welch"),
            Self::Sin => write!(f, "Sin"),
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

type FilterFn = fn(usize, usize, f64, f64, f64) -> f64;

fn filter_low_pass(n: usize, shift: usize, dt: f64, _f_lo_cut: f64, f_hi_cut: f64) -> f64 {
    let n = n as f64;
    let shift = shift as f64;

    if n != shift {
        (2.0 * PI * f_hi_cut * dt * (n - shift)).sin() / (PI * dt * (n - shift))
    } else {
        2.0 * f_hi_cut
    }
}

fn filter_high_pass(n: usize, shift: usize, dt: f64, f_lo_cut: f64, _f_hi_cut: f64) -> f64 {
    let n = n as f64;
    let shift = shift as f64;

    if n != shift {
        ((PI * (n - shift)).sin() - (2.0 * PI * f_lo_cut * dt * (n - shift)).sin())
            / (PI * dt * (n - shift))
    } else {
        1.0 / dt - 2.0 * f_lo_cut
    }
}

fn filter_band_pass(n: usize, shift: usize, dt: f64, f_lo_cut: f64, f_hi_cut: f64) -> f64 {
    let n = n as f64;
    let shift = shift as f64;

    if n != shift {
        ((2.0 * PI * f_hi_cut * dt * (n - shift)).sin()
            - (2.0 * PI * f_lo_cut * dt * (n - shift)).sin())
            / (PI * dt * (n - shift))
    } else {
        2.0 * f_hi_cut - 2.0 * f_lo_cut
    }
}

fn filter_band_stop(n: usize, shift: usize, dt: f64, f_lo_cut: f64, f_hi_cut: f64) -> f64 {
    let n = n as f64;
    let shift = shift as f64;

    if n != shift {
        ((2.0 * PI * f_lo_cut * dt * (n - shift)).sin()
            - (2.0 * PI * f_hi_cut * dt * (n - shift)).sin()
            + (PI * (n - shift)).sin())
            / (PI * dt * (n - shift))
    } else {
        2.0 * f_lo_cut - 2.0 * f_hi_cut + 1.0 / dt
    }
}

#[derive(Default, PartialEq, Clone)]
pub enum Filter {
    #[default]
    LowPass,
    HighPass,
    BandPass,
    BandStop,
}

impl Filter {
    fn function(&self) -> FilterFn {
        match self {
            Self::LowPass => filter_low_pass,
            Self::HighPass => filter_high_pass,
            Self::BandPass => filter_band_pass,
            Self::BandStop => filter_band_stop,
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LowPass => write!(f, "Low Pass"),
            Self::HighPass => write!(f, "High Pass"),
            Self::BandPass => write!(f, "Band Pass"),
            Self::BandStop => write!(f, "Band Stop"),
        }
    }
}
