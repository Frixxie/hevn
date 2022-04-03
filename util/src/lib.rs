mod util;


pub use crate::util::{
    Collector, CollectorError, EnvData, ShellyS1, ShellyS1Error, ShellyStatus, SmartAppliance,
};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
