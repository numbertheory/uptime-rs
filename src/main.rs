use std::fs::File;
use std::io::prelude::Read;
use std::io::prelude::Seek;
use std::io::SeekFrom;
use std::time::Duration;
use std::env;
use indoc::formatdoc;
use itertools::Itertools;

fn main() -> std::io::Result<()> {
    let uptime = get_uptime()?;
    let load = get_load_avg()?;
    let count = get_users()?;
    let args: Vec<String> = env::args().collect();
    let mut days = 0;
    let mut weeks = 0;
    let mut months = 0;
    let mut years = 0;
    let mut centuries = 0;
    let mut friendly_hours = String::new();
    let mut friendly_days = String::new();
    let mut friendly_weeks = String::new();
    if args.len() > 1 {
        let output_format = &args[1];
        if uptime.0 > 24 {
            days = uptime.0 / 24;
            if uptime.0 % 24 != 0 {
                friendly_hours = format!("{} hours, ", (uptime.0 % 24).to_string());
            }
        }
        if days >= 7 {
            weeks = days / 7;
            if days % 7 != 0 {
                if days % 7 == 1 {
                    friendly_days = format!("{} day, ", (days % 7).to_string());
                } else {
                    friendly_days = format!("{} days, ", (days % 7).to_string());
                }
            }
            if weeks > 0 {
                if weeks == 1 {
                    friendly_weeks = format!("{} week, ", weeks.to_string());
                } else {
                    friendly_weeks = format!("{} weeks, ", weeks.to_string());
                }
            }
        }
        if days >= 30 {
            months = days / 30;
        }
        if days >= 365 {
            years = days / 3;
        }
        if years >= 100 {
            centuries = years / 100;
        }
        if output_format == "--json" {
            let friendly_format = formatdoc!(r#"{friendly_weeks}{friendly_days}{friendly_hours}"#,
                friendly_hours = friendly_hours,
                friendly_days = friendly_days,
                friendly_weeks = friendly_weeks);
            let json_line = formatdoc!(r#"
            {{
                "days": {days},
                "weeks": {weeks},
                "months": {months},
                "years": {years},
                "centuries": {centuries},
                "friendly_format": "{friendly_format}"
            }}"#, 
                days = days,
                weeks = weeks,
                months = months,
                years = years,
                centuries = centuries,
                friendly_format = friendly_format.trim().trim_end_matches(",")
                );

            println!("{}", json_line)
        }
    } else {
        println!(
            "uptime: {}h:{}m:{}s, load average: {} {} {}, users: {}",
            uptime.0, uptime.1, uptime.2, load.0, load.1, load.2, count
        );
    }
    
    Ok(())
}

fn get_uptime() -> std::io::Result<(u64, u64, u64)> {
    let mut file = File::open("/proc/uptime")?;
    let mut c = String::new();
    file.read_to_string(&mut c)?;
    let uptime = Duration::from_secs_f32(
        c.split_whitespace()
            .next()
            .unwrap_or("0")
            .parse::<f32>()
            .unwrap_or(0.0),
    );
    let h = uptime.as_secs() / 3600;
    let m = (uptime.as_secs() - h * 3600) / 60;
    let s = uptime.as_secs() - h * 3600 - m * 60;
    Ok((h, m, s))
}

fn get_load_avg() -> std::io::Result<(f32, f32, f32)> {
    let mut file = File::open("/proc/loadavg")?;
    let mut c = String::new();
    file.read_to_string(&mut c)?;
    let avg = c
        .split_whitespace() // split at whitespace
        .take(3) // take the first 3 values
        .map(|s| s.parse::<f32>().unwrap_or(0.0)) // convert each value to f32
        .collect::<Vec<f32>>(); // collect into vector
    Ok((avg[0], avg[1], avg[2]))
}

fn get_users() -> std::io::Result<u32> {
    let mut file = File::open("/var/run/utmp")?;
    let mut count = 0;
    for i in 2..=file.metadata()?.len() / 384 {
        file.seek(SeekFrom::Start(384 * i + 42 + 2))?;
        let mut data: [u8; 30] = [0; 30];
        file.read(&mut data)?;
        if data != [0; 30] {
            count = count + 1;
        }
    }
    Ok(count)
}
