#![feature(rustc_private)]
#![feature(core)]
#![feature(io)]
extern crate hyper;
extern crate serialize;

use std::env;
use std::io::Read;

use serialize::{json, Decodable, Decoder};

use hyper::Client;
use hyper::header::Connection;
use hyper::header::ConnectionOption;
use hyper::client::response::Response as Response;
use hyper::HttpError as HttpError;

#[allow(dead_code)]
fn main(){
    match read_configuration() {
        Ok(config) =>
            match ask_api_for_weather(&config) {
                Ok(current_conditions) => println!("It is: {}", current_conditions.icon),
                Err(error) => println!("Something went wrong: {}", error.message),
            },
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

fn ask_api_for_weather(configuration : &Configuration) -> Result<CurrentWeatherConditions, ProgramError> {
    let mut client = Client::new();

    let response : Result<Response, HttpError> = client.get(configuration.to_url().as_slice())
        .header(Connection(vec![ConnectionOption::Close]))
        .send();

    match response {
        Ok(mut res) => {
            let mut body = String::new();
            res.read_to_string(&mut body).unwrap();

            Ok(parse_out_current_conditions(&body))
        },
        Err(_) => Err(ProgramError { message: "HTTP request failed".to_string() })
    }
}

#[derive(Debug)]
struct CurrentWeatherConditions {
    icon: String,
}

impl Decodable for CurrentWeatherConditions {
    fn decode<S: Decoder>(decoder: &mut S) -> Result<CurrentWeatherConditions, S::Error> {
        decoder.read_struct("root", 0, |decoder| {
            decoder.read_struct_field("currently", 0, |decoder| {
                Ok(CurrentWeatherConditions{
                    icon: try!(decoder.read_struct_field("icon", 0, |decoder| Decodable::decode(decoder))),
                })
            })
        })
    }
}

fn parse_out_current_conditions(raw_json : &str) -> CurrentWeatherConditions {
    let wc : CurrentWeatherConditions = json::decode(raw_json).unwrap();
    wc
}

#[cfg(test)]
mod tests {
    use std::env;
    use read_configuration;
    use parse_out_current_conditions;
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

    #[test]
    fn test_parse_json(){
        let json = r##"{"latitude":37.8267,"longitude":-122.423,"timezone":"America/Los_Angeles","offset":-7,"currently":{"time":1425827066,"summary":"Partly Cloudy","icon":"partly-cloudy-day","nearestStormDistance":3,"nearestStormBearing":37,"precipIntensity":0,"precipProbability":0,"temperature":50.87,"apparentTemperature":50.87,"dewPoint":48.98,"humidity":0.93,"windSpeed":3.09,"windBearing":291,"visibility":4.19,"cloudCover":0.53,"pressure":1018.05,"ozone":311.83}}"##;
        assert_eq!(parse_out_current_conditions(json).icon, "partly-cloudy-day".to_string());
    }
}
