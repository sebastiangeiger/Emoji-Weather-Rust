use std::env;

fn main(){

}

#[derive(Debug, Eq, PartialEq)]
struct ProgramError {
    message: String,
}

#[derive(Debug, Eq, PartialEq)]
struct Configuration {
    api_key: String,
    lat: String,
    lng: String,
}

type ConfigurationResult = Result<Configuration, ProgramError>;

fn read_configuration() -> ConfigurationResult {
    let envs = (env::var("FORECAST_IO_API_KEY"), env::var("MY_LAT"), env::var("MY_LNG"));
    match envs {
        (Ok(api_key), Ok(lat), Ok(lng)) =>
            Ok(Configuration { api_key: api_key, lat: lat, lng: lng }),

        (Err(_), _, _) =>
            Err(ProgramError { message: "ENV['FORECAST_IO_API_KEY'] not set".to_string() }),

        (_, Err(_), _) =>
            Err(ProgramError { message: "ENV['MY_LAT'] not set".to_string() }),

        (_, _, Err(_)) =>
            Err(ProgramError { message: "ENV['MY_LNG'] not set".to_string() }),
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use read_configuration;
    use Configuration;
    use ProgramError;

    #[test]
    fn test_read_configuration() {
        env::set_var("FORECAST_IO_API_KEY", "123-KEY-456");
        env::set_var("MY_LAT", "33.835297");
        env::set_var("MY_LNG", "-84.321231");
        let config = Configuration {
            api_key: "123-KEY-456".to_string(),
            lat: "33.835297".to_string(),
            lng: "-84.321231".to_string(),
        };
        assert_eq!(read_configuration(), Ok(config));

        env::remove_var("MY_LNG");
        let error = ProgramError { message: "ENV['MY_LNG'] not set".to_string() };
        assert_eq!(read_configuration(), Err(error));

        env::remove_var("MY_LAT");
        let error = ProgramError { message: "ENV['MY_LAT'] not set".to_string() };
        assert_eq!(read_configuration(), Err(error));

        env::remove_var("FORECAST_IO_API_KEY");
        let error = ProgramError { message: "ENV['FORECAST_IO_API_KEY'] not set".to_string() };
        assert_eq!(read_configuration(), Err(error));
    }
}
