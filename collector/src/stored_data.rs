use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use util::EnvData;

pub trait Stats:
    std::ops::Add<Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::Mul<Output = Self>
    + std::ops::Sub<Output = Self>
    + Copy
    + Default
    + std::fmt::Debug
    + std::convert::From<u16>
    + std::cmp::PartialOrd
{
}

impl Stats for f32 {}
impl Stats for i32 {}

pub fn mean<T: Stats>(values: &[T]) -> Option<T> {
    if values.is_empty() {
        return None;
    }
    Some(values.iter().fold(T::default(), |a, b| a + *b) / (values.len() as u16).into())
}

pub fn std_dev<T: Stats>(values: &[T], mean: T) -> Option<T> {
    if values.is_empty() {
        return None;
    }
    Some(
        values.iter().fold(T::default(), |a, b| {
            let val = *b - mean;
            a + val * val
        }) / (values.len() as u16 - 1).into(),
    )
}

pub fn linear_regression<T: Stats>(xs: &[T], ys: &[T]) -> Option<(T, T)> {
    let xy = xs
        .iter()
        .zip(ys)
        .map(|(x, y)| (*x * *y))
        .collect::<Vec<T>>();

    let x2 = xs.iter().map(|x| (*x * *x)).collect::<Vec<T>>();

    let xy_mean = mean(&xy)?;
    let x_mean = mean(xs)?;
    let y_mean = mean(ys)?;
    let x2_mean = mean(&x2)?;

    let slope = (xy_mean - x_mean * y_mean) / (x2_mean - x_mean * x_mean);
    let intercept = y_mean - slope * x_mean;

    dbg!(slope, intercept);
    Some((slope, intercept))
}

pub struct StoredData {
    s_data: Mutex<VecDeque<(Duration, EnvData)>>,
    lim: usize,
    p_start: Instant,
}

impl StoredData {
    pub fn new(lim: usize) -> Self {
        StoredData {
            s_data: Mutex::new(VecDeque::new()),
            lim,
            p_start: Instant::now(),
        }
    }

    pub async fn add(&self, data: EnvData) {
        let mut s_data = self.s_data.lock().await;
        s_data.push_back((self.p_start.elapsed(), data));
    }

    pub async fn remove(&self) -> Option<(Duration, EnvData)> {
        let mut s_data = self.s_data.lock().await;
        s_data.pop_front()
    }

    pub async fn len(&self) -> usize {
        let s_data = self.s_data.lock().await;
        s_data.len()
    }

    pub fn get_lim(&self) -> usize {
        self.lim
    }

    pub async fn get_expected_deviation<T: Stats + From<i16>>(&self, factor: T) -> Option<(T, T)> {
        let s_data = self.s_data.lock().await;
        let len = s_data.len();
        if len < self.lim {
            return None;
        }
        let humis = s_data
            .iter()
            .map(|(_, v)| (v.humidity.into()))
            .collect::<Vec<T>>();
        let temps = s_data
            .iter()
            .map(|(_, v)| (v.temperature.into()))
            .collect::<Vec<T>>();

        let mut std_temp: T = std_dev(&temps, mean(&temps)?)?;
        let mut std_humi: T = std_dev(&humis, mean(&humis)?)?;

        if std_temp < 10i16.into() {
            std_temp = 10i16.into();
        }
        if std_humi < 20i16.into() {
            std_humi = 20i16.into();
        }

        Some((std_temp * factor.into(), std_humi * factor.into()))
    }

    /// predict the temperature and humidity
    /// based on the last 5 values
    /// using linear regression
    pub async fn predict(&self) -> Option<EnvData> {
        let s_data = self.s_data.lock().await;

        if s_data.len() < self.lim {
            return None;
        }

        for (i, data) in s_data.iter().enumerate() {
            dbg!(i, data);
        }

        // let x = (0..s_data.len()).map(|x| x as f32).collect::<Vec<f32>>();
        let x = s_data
            .iter()
            .map(|(t, _)| t.as_secs_f32())
            .collect::<Vec<f32>>();
        let humis = s_data
            .iter()
            .map(|(_, v)| (v.humidity as f32))
            .collect::<Vec<_>>();
        let temps = s_data
            .iter()
            .map(|(_, v)| (v.temperature as f32))
            .collect::<Vec<_>>();

        let res_humi = linear_regression(&x, &humis)?;
        let res_temp = linear_regression(&x, &temps)?;

        let predicted_humi = res_humi.0 * self.p_start.elapsed().as_secs_f32() + res_humi.1;
        let predicted_temp = res_temp.0 * self.p_start.elapsed().as_secs_f32() + res_temp.1;

        Some(EnvData::new(
            s_data[0].1.room.clone(),
            predicted_temp as i16,
            predicted_humi as u16,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_linear_regression() {
        let xs = [0, 1, 2, 3, 4];
        let ys = [-2, 0, 2, 4, 6];

        let res = linear_regression(&xs, &ys);
        assert_eq!(res.unwrap(), (2, -2));
    }

    #[test]
    fn is_mean() {
        let xs = [1, 2, 3, 4, 5];
        let ys = [2, 4, 6, 8, 10];

        let res = mean(&xs);
        assert_eq!(res.unwrap(), 3);
        let res = mean(&ys);
        assert_eq!(res.unwrap(), 6);
    }
}
