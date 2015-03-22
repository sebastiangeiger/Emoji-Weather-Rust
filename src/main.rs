#![feature(rustc_private)]
#![feature(core)]
#![feature(io)]
#![feature(path)]
extern crate hyper;
extern crate serialize;

use std::env;
use std::io::{Read, Write};
use std::num::Float;
use std::fs::File;
use std::path::Path;

use serialize::{json, Decodable, Decoder};

use hyper::{Client, HttpError};
use hyper::header::{Connection, ConnectionOption};
use hyper::status::StatusCode;
use hyper::client::response::Response;


macro_rules! println_stderr(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);


#[allow(dead_code)]
fn main(){
    match ask_api_for_weather() {
        Ok(current_conditions) => {
            let emoji = WeatherIcons::new(current_conditions.icon.as_slice()).to_emoji();
            println!("{}°/{}°{}", current_conditions.temperature_in_celsius(), current_conditions.temperature.round(), emoji)
        },
        Err(error) => println_stderr!("Something went wrong: {}", error.message),
    }
}

fn ask_api_for_weather() -> Result<CurrentWeatherConditions, ProgramError> {
    let path = Path::new("/Users/seb/.weather.conf");
    let configuration = try!(read_configuration(&path));
    let body = try!(get_request(&configuration.to_url()));
    let weather_conditions = try!(parse_out_current_conditions(body.as_slice()));
    Ok(weather_conditions)
}

#[derive(Debug, Eq, PartialEq)]
enum WeatherIcons {
    ClearDay,
    ClearNight,
    Rain,
    Snow,
    Sleet,
    Wind,
    Fog,
    Cloudy,
    PartlyCloudyDay,
    PartlyCloudyNight,
    Unknown
}

impl WeatherIcons {
    fn new(api_icon_name : &str) -> WeatherIcons {
        match api_icon_name {
            "clear-day"           => WeatherIcons::ClearDay,
            "clear-night"         => WeatherIcons::ClearNight,
            "rain"                => WeatherIcons::Rain,
            "snow"                => WeatherIcons::Snow,
            "sleet"               => WeatherIcons::Sleet,
            "wind"                => WeatherIcons::Wind,
            "fog"                 => WeatherIcons::Fog,
            "cloudy"              => WeatherIcons::Cloudy,
            "partly-cloudy-day"   => WeatherIcons::PartlyCloudyDay,
            "partly-cloudy-night" => WeatherIcons::PartlyCloudyNight,
            _                     => WeatherIcons::Unknown
        }
    }

    fn to_emoji(&self) -> String {
        let result = match *self {
            WeatherIcons::ClearDay          => "🌞 ",
            WeatherIcons::ClearNight        => "🌚 ",
            WeatherIcons::Rain              => "☔️ ",
            WeatherIcons::Snow              => "⛄️ ",
            WeatherIcons::Sleet             => "☔️❄️ ",
            WeatherIcons::Wind              => "💨 ",
            WeatherIcons::Fog               => "🌁 ",
            WeatherIcons::Cloudy            => "☁️ ",
            WeatherIcons::PartlyCloudyDay   => "⛅️ ",
            WeatherIcons::PartlyCloudyNight => "🌚☁️ ",
            WeatherIcons::Unknown           => "❓ ",
        };
        result.to_string()
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

fn read_configuration(path : &Path) -> ConfigurationResult {
    let lat = try!(read_from_file(path, "LAT"));
    let lng = try!(read_from_file(path, "LNG"));
    match env::var("FORECAST_IO_API_KEY") {
        Ok(api_key) => Ok(Configuration { api_key: api_key, lat: lat, lng: lng }),
        Err(_) => Err(ProgramError { message: "ENV['FORECAST_IO_API_KEY'] not set".to_string() })
    }
}


fn read_from_file(path : &Path, key : &str) -> Result<String, ProgramError> {
    let contents = try!(read_file(path));
    for line in contents.lines() {
        let fragments : Vec<&str> = line.split("=").collect();
        if fragments[0] == key {
            return Ok(fragments[1].to_string())
        }
    }
    Err(ProgramError { message: format!("Could not find '{}' in '{}'", key, path.display()) })
}

fn read_file(path : &Path) -> Result<String, ProgramError> {
    let mut result = String::new();
    match File::open(path) {
        Ok(mut file) => {
            match file.read_to_string(&mut result) {
                Err(_) => Err(ProgramError { message: "Could not read configuration file".to_string() }),
                Ok(_) => Ok(result),
            }
        },
        Err(_) => Err(ProgramError { message: format!("Configuration file '{}' does not exist", path.display()) })
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

fn get_request(url : &Url) -> Result<String, ProgramError>{
    let mut client = Client::new();

    let response : Result<Response, HttpError> = client.get(url.as_slice())
        .header(Connection(vec![ConnectionOption::Close]))
        .send();

    match response {
        Ok(mut res) => {
            let mut body = String::new();
            match res.status {
                StatusCode::Ok => {
                    match res.read_to_string(&mut body) {
                        Ok(_) => Ok(body),
                        Err(_) => Err(ProgramError { message: "HTTP request failed".to_string() }),
                    }
                },
                _ => {
                    let mut message = "HTTP request failed, server returned ".to_string();
                    message.push_str(res.status.canonical_reason().unwrap());
                    Err(ProgramError { message: message })
                }
            }
        },
        Err(_) => Err(ProgramError { message: "HTTP request failed, could not reach server".to_string() })
    }
}

#[derive(Debug)]
struct CurrentWeatherConditions {
    icon: String,
    temperature: f32,
}

impl Decodable for CurrentWeatherConditions {
    fn decode<S: Decoder>(decoder: &mut S) -> Result<CurrentWeatherConditions, S::Error> {
        decoder.read_struct("root", 0, |decoder| {
            decoder.read_struct_field("currently", 0, |decoder| {
                let icon = try!(decoder.read_struct_field("icon", 0, |decoder| Decodable::decode(decoder)));
                let temperature = try!(decoder.read_struct_field("temperature", 0, |decoder| Decodable::decode(decoder)));
                Ok(CurrentWeatherConditions{
                    temperature: temperature,
                    icon: icon,
                })
            })
        })
    }
}

impl CurrentWeatherConditions {
    fn temperature_in_celsius(&self) -> i32 {
        ((self.temperature - 32.0) * 5.0 / 9.0).round() as i32
    }
}

fn parse_out_current_conditions(raw_json : &str) -> Result<CurrentWeatherConditions, ProgramError> {
    let wc : Result<CurrentWeatherConditions, serialize::json::DecoderError> = json::decode(raw_json);
    match wc {
        Ok(wc) => Ok(wc),
        Err(error) =>{
            println!("{}", error);
            Err(ProgramError { message: "JSON parsing failed".to_string() })
        }
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::path::Path;
    use read_from_file;
    use read_configuration;
    use parse_out_current_conditions;
    use Configuration;
    use WeatherIcons;
    use ProgramError;
    use CurrentWeatherConditions;

    #[test]
    fn test_read_configuration() {
        env::set_var("FORECAST_IO_API_KEY", "123-KEY-456");
        env::remove_var("MY_LNG");
        env::remove_var("MY_LAT");
        let config_file = Path::new("weather.conf.example");

        let config = Configuration {
            api_key: "123-KEY-456".to_string(),
            lat: "33.825553".to_string(),
            lng: "-84.338453".to_string(),
        };
        assert_eq!(read_configuration(&config_file), Ok(config));

        let some_file = Path::new("some_file");
        let file_error = ProgramError { message: "Configuration file 'some_file' does not exist".to_string() };
        assert_eq!(read_configuration(&some_file), Err(file_error));

        env::remove_var("FORECAST_IO_API_KEY");
        let env_error = ProgramError { message: "ENV['FORECAST_IO_API_KEY'] not set".to_string() };
        assert_eq!(read_configuration(&config_file), Err(env_error));
    }

    #[test]
    fn test_read_from_file(){
        let existing_file_path = Path::new("weather.conf.example");
        assert_eq!(read_from_file(&existing_file_path, "LAT"), Ok("33.825553".to_string()));

        let non_existing_file_path = Path::new("something_else.conf");
        let does_not_exist = ProgramError { message: "Configuration file 'something_else.conf' does not exist".to_string() };
        assert_eq!(read_from_file(&non_existing_file_path, "LAT"), Err(does_not_exist));

        let not_in_file = ProgramError { message: "Could not find 'GARBAGE' in 'weather.conf.example'".to_string() };
        assert_eq!(read_from_file(&existing_file_path, "GARBAGE"), Err(not_in_file));
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
        assert_eq!(parse_out_current_conditions(json).unwrap().icon, "partly-cloudy-day".to_string());
        assert_eq!(parse_out_current_conditions(json).unwrap().temperature, 50.87);
    }

    #[test]
    fn test_temperature_in_celcius(){
        let conditions = CurrentWeatherConditions {
            icon: "partly-cloudy-day".to_string(),
            temperature: 50.87
        };
        assert_eq!(conditions.temperature_in_celsius(), 10);
    }

    #[test]
    fn test_weather_icon_constructor(){
        assert_eq!(WeatherIcons::new("clear-day"), WeatherIcons::ClearDay);
        assert_eq!(WeatherIcons::new("clear-night"), WeatherIcons::ClearNight);
    }

    #[test]
    fn test_weather_icon_to_emoji(){
        assert_eq!(WeatherIcons::ClearDay.to_emoji(), "🌞 ".to_string());
        assert_eq!(WeatherIcons::ClearNight.to_emoji(), "🌚 ".to_string());
    }
}
