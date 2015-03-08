use std::env;

fn main(){

}

#[derive(Debug, Eq, PartialEq)]
struct Configuration {
    api_key: String,
}

#[derive(Debug, Eq, PartialEq)]
struct ProgramError {
    message: String,
}

type ConfigurationResult = Result<Configuration, ProgramError>;

fn read_configuration() -> ConfigurationResult {
    match env::var("FORECAST_IO_API_KEY") {
        Ok(api_key) => Ok(Configuration { api_key: api_key }),
        Err(_) => Err(ProgramError { message: "ENV['FORECAST_IO_API_KEY'] not set".to_string() })
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use read_configuration;
    use Configuration;
    use ProgramError;

    #[test]
    fn test_configuration_equality() {
        let config_1 = Configuration { api_key: "A".to_string() };
        let config_2 = Configuration { api_key: "A".to_string() };
        let config_3 = Configuration { api_key: "B".to_string() };
        assert_eq!(config_1, config_1);
        assert_eq!(config_1, config_2);
        assert!(config_1 != config_3);
    }

    #[test]
    fn test_read_configuration() {
        env::set_var("FORECAST_IO_API_KEY", "123-KEY-456");
        let config = Configuration { api_key: "123-KEY-456".to_string() };
        assert_eq!(read_configuration(), Ok(config));

        env::remove_var("FORECAST_IO_API_KEY");
        let error = ProgramError { message: "ENV['FORECAST_IO_API_KEY'] not set".to_string() };
        assert_eq!(read_configuration(), Err(error));
    }
}
