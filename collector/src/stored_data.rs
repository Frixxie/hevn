use tokio::sync::Mutex;
use util::EnvData;
use std::collections::VecDeque;

pub fn mean<T>(values: &[T]) -> Option<T>
where
    T: std::ops::Add<Output = T> + std::ops::Div<Output = T> + From<u16> + Default + Copy,
{
    if values.is_empty() {
        return None;
    }
    Some(values.iter().fold(T::default(), |a, b| a + *b) / (values.len() as u16).into())
}

pub fn std_dev<T>(values: &[T], mean: T) -> Option<T>
where
    T: std::ops::Sub<Output = T>
        + std::ops::Add<Output = T>
        + std::ops::Mul<Output = T>
        + std::ops::Div<Output = T>
        + From<u16>
        + Default
        + Copy,
{
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

pub fn linear_regression<T>(xs: &[T], ys: &[T]) -> Option<(T, T)>
where
    T: std::ops::Add<Output = T>
        + std::ops::Div<Output = T>
        + std::ops::Mul<Output = T>
        + std::ops::Sub<Output = T>
        + Copy
        + Default
        + std::fmt::Debug
        + std::convert::From<u16>,
{
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
    s_data: Mutex<VecDeque<EnvData>>,
    lim: usize,
}

impl StoredData {
    pub fn new(lim: usize) -> Self {
        StoredData {
            s_data: Mutex::new(VecDeque::new()),
            lim,
        }
    }

    pub async fn add(&self, data: EnvData) {
        let mut s_data = self.s_data.lock().await;
        s_data.push_back(data);
    }

    pub async fn remove(&self) -> Option<EnvData> {
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

    pub async fn get_expected_deviation<T>(&self, factor: T) -> Option<(T, T)>
    where
        T: From<u16>
            + From<i16>
            + std::ops::Sub<Output = T>
            + std::ops::Add<Output = T>
            + std::ops::Mul<Output = T>
            + std::ops::Div<Output = T>
            + Default
            + Copy,
    {
        let s_data = self.s_data.lock().await;
        let len = s_data.len();
        if len < self.lim {
            return None;
        }
        let humis = s_data
            .iter()
            .map(|v| (v.humidity.into()))
            .collect::<Vec<T>>();
        let temps = s_data
            .iter()
            .map(|v| (v.temperature.into()))
            .collect::<Vec<T>>();

        Some((
            std_dev(&temps, mean(&temps)?)? * factor,
            std_dev(&humis, mean(&humis)?)? * factor,
        ))
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
            println!("{},{}", i, data);
        }

        let x = (0..s_data.len()).map(|x| x as f32).collect::<Vec<f32>>();
        let humis = s_data
            .iter()
            .map(|v| (v.humidity as f32))
            .collect::<Vec<_>>();
        let temps = s_data
            .iter()
            .map(|v| (v.temperature as f32))
            .collect::<Vec<_>>();

        let res_humi = linear_regression(&x, &humis)?;
        let res_temp = linear_regression(&x, &temps)?;

        let predicted_humi = res_humi.0 * (x.len()) as f32 + res_humi.1;
        let predicted_temp = res_temp.0 * (x.len()) as f32 + res_temp.1;

        Some(EnvData::new(
            s_data[0].room.clone(),
            predicted_temp as i16,
            predicted_humi as u16,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_regression() {
        let xs = [0, 1, 2, 3, 4];
        let ys = [-2, 0, 2, 4, 6];

        let res = linear_regression(&xs, &ys);
        assert_eq!(res.unwrap(), (2, -2));
    }

    #[test]
    fn test_mean() {
        let xs = [1, 2, 3, 4, 5];
        let ys = [2, 4, 6, 8, 10];

        let res = mean(&xs);
        assert_eq!(res.unwrap(), 3);
        let res = mean(&ys);
        assert_eq!(res.unwrap(), 6);
    }

    #[tokio::test]
    async fn test_stored_data_add_remove() {
        let s_data = StoredData::new(5);
        let data = EnvData::new("test".to_string(), 10, 20);
        s_data.add(data.clone()).await;
        let res = s_data.remove().await.unwrap();
        assert_eq!(res, data);
        assert_eq!(s_data.len().await, 0);
    }

    #[tokio::test]
    async fn test_stored_data_predict_one() {
        let s_data = StoredData::new(5);
        let data = EnvData::new("test".to_string(), 10, 20);
        s_data.add(data.clone()).await;
        let res = s_data.predict().await;
        assert_eq!(res, None);
    }

    #[tokio::test]
    async fn test_stored_data_predict_many_increase() {
        let s_data = StoredData::new(5);
        for i in 0..5 {
            let data = EnvData::new("test".to_string(), i * 2, (i as i16).try_into().unwrap());
            s_data.add(data.clone()).await;
        }
        let res = s_data.predict().await;
        assert_eq!(res, Some(EnvData::new("test".to_string(), 10, 5)));
    }

    #[tokio::test]
    async fn test_stored_data_predict_many_decrease() {
        let s_data = StoredData::new(5);
        for i in (5..10).into_iter().rev() {
            let data = EnvData::new("test".to_string(), i, (i as i16).try_into().unwrap());
            s_data.add(data.clone()).await;
        }
        let res = s_data.predict().await;
        assert_eq!(res, Some(EnvData::new("test".to_string(), 4, 4)));
    }

    #[tokio::test]
    async fn test_stored_data_predict_many_same() {
        let s_data = StoredData::new(5);
        for _ in 0..5 {
            let data = EnvData::new("test".to_string(), 5, 5.try_into().unwrap());
            s_data.add(data.clone()).await;
        }
        let res = s_data.predict().await;
        assert_eq!(res, Some(EnvData::new("test".to_string(), 5, 5)));
    }
}
