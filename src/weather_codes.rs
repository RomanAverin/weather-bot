use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum WeatherCode {
    ClearSky,
    Cloudy,
    Fair,
    Fog,
    HeavyRain,
    HeavyRainAndThunder,
    HeavyRainShowers,
    HeavyRainShowersAndThunder,
    HeavySleet,
    HeavySleetAndThunder,
    HeavySleetShowers,
    HeavySleetShowersAndThunder,
    HeavySnow,
    HeavySnowAndThunder,
    HeavySnowShowers,
    HeavySnowShowersAndThunder,
    LightRain,
    LightRainAndThunder,
    LightRainShowers,
    LightRainShowersAndThunder,
    LightSleet,
    LightSleetAndThunder,
    LightSleetShowers,
    LightSnow,
    LightSnowAndThunder,
    LightSnowShowers,
    LightsSleetShowersAndThunder,
    LightsSnowShowersAndThunder,
    PartlyCloudy,
    Rain,
    RainAndThunder,
    RainShowers,
    RainShowersAndThunder,
    Sleet,
}

pub fn get_weather_code(string_code: &str) -> WeatherCode {
    let mut weather_codes = HashMap::new();
    weather_codes.insert("clearsky", WeatherCode::ClearSky);
    weather_codes.insert("cloudy", WeatherCode::Cloudy);
    weather_codes.insert("fair", WeatherCode::Fair);
    weather_codes.insert("fog", WeatherCode::Fog);
    weather_codes.insert("heavyrain", WeatherCode::HeavyRain);
    weather_codes.insert("heavyrainandthunder", WeatherCode::HeavyRainAndThunder);
    weather_codes.insert("heavyrainshowers", WeatherCode::HeavyRainShowers);
    weather_codes.insert(
        "heavyrainshowersandthunder",
        WeatherCode::HeavyRainShowersAndThunder,
    );
    weather_codes.insert("heavysleet", WeatherCode::HeavySleet);
    weather_codes.insert("heavysleetandthunder", WeatherCode::HeavySleetAndThunder);
    weather_codes.insert("heavysleetshowers", WeatherCode::HeavySleetShowers);
    weather_codes.insert(
        "heavysleetshowersandthunder",
        WeatherCode::HeavySleetShowersAndThunder,
    );
    weather_codes.insert("heavysnow", WeatherCode::HeavySnow);
    weather_codes.insert("heavysnowandthunder", WeatherCode::HeavySnowAndThunder);
    weather_codes.insert("heavysnowshowers", WeatherCode::HeavySnowShowers);
    weather_codes.insert(
        "heavysnowshowersandthunder",
        WeatherCode::HeavySnowShowersAndThunder,
    );
    weather_codes.insert("lightrain", WeatherCode::LightRain);
    weather_codes.insert("lightrainshowers", WeatherCode::LightRainShowers);
    weather_codes.insert(
        "lightrainshowersandthunder",
        WeatherCode::LightRainShowersAndThunder,
    );
    weather_codes.insert("lightsleet", WeatherCode::LightSleet);
    weather_codes.insert("lightsleetandthunder", WeatherCode::LightSleetAndThunder);
    weather_codes.insert("lightsleetshowers", WeatherCode::LightSleetShowers);
    weather_codes.insert("lightsnowandthunder", WeatherCode::LightSnowAndThunder);
    weather_codes.insert("lightsnowshowers", WeatherCode::LightSnowShowers);
    weather_codes.insert(
        "lightssleetshowersandthunder",
        WeatherCode::LightsSleetShowersAndThunder,
    );
    weather_codes.insert(
        "lightssnowshowersandthunder",
        WeatherCode::LightsSnowShowersAndThunder,
    );
    weather_codes.insert("partlycloudy", WeatherCode::PartlyCloudy);
    weather_codes.insert("rain", WeatherCode::Rain);
    weather_codes.insert("rainandthunder", WeatherCode::RainAndThunder);
    weather_codes.insert("rainshowers", WeatherCode::RainShowers);
    weather_codes.insert("rainshowersandthunder", WeatherCode::RainShowersAndThunder);
    weather_codes.insert("sleet", WeatherCode::Sleet);

    if let Some(code) = weather_codes.get(string_code) {
        *code
    } else {
        WeatherCode::ClearSky
    }
}
