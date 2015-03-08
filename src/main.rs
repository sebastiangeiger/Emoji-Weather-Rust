#![feature(core)]
#![feature(io)]
extern crate hyper;

use std::env;
use std::io::Read;

use hyper::Client;
use hyper::header::Connection;
use hyper::header::ConnectionOption;
use hyper::client::response::Response as Response;
use hyper::HttpError as HttpError;

fn main(){
    match read_configuration() {
        Ok(config) => ask_api_for_weather(&config),
        Err(error) => println!("Something went wrong: {}", error.message)
    }
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

type Url = String;

impl Configuration {
    fn to_url(&self) -> Url {
        let mut result : Url = "https://api.forecast.io/forecast/".to_string();
        result.push_str(self.api_key.as_slice());
        result.push_str("/");
        result.push_str(self.lat.as_slice());
        result.push_str(",");
        result.push_str(self.lng.as_slice());
        result
    }
}

fn ask_api_for_weather(configuration : &Configuration) {
    let mut client = Client::new();

    let response : Result<Response, HttpError> = client.get(configuration.to_url().as_slice())
        .header(Connection(vec![ConnectionOption::Close]))
        .send();

    match response {
        Ok(mut res) => {
            let mut body = String::new();
            res.read_to_string(&mut body).unwrap();

            println!("Response: {}", body);
        },
        Err(_) => println!("Something went wrong")
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

    #[test]
    fn test_configuration_to_url(){
        let config = Configuration {
            api_key: "123-KEY-456".to_string(),
            lat: "33.835297".to_string(),
            lng: "-84.321231".to_string(),
        };
        assert_eq!(config.to_url(), "https://api.forecast.io/forecast/123-KEY-456/33.835297,-84.321231".to_string());
    }
}
