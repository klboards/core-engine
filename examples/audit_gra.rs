//! Audit: GRA proportional-day basis for jerusalem-old-city, 2026-06-21.
use core_engine::events::{read_jd, Direction, ReadSpec};
use core_engine::optics::{HorizonMode, LimbReference, RefractionModel};
use core_engine::params::Optics;
use core_engine::time::jd_from_gregorian;
use core_engine::Site;

fn jd_to_local(jd: f64, lon_deg: f64) -> String {
    // wall clock at +2h (Asia/Jerusalem DST = +3 on this date). Use +3 for IDT June.
    let day_frac = (jd + 0.5).fract();
    let secs_utc = day_frac * 86400.0;
    let secs_local = secs_utc + 3.0 * 3600.0; // IDT
    let s = ((secs_local % 86400.0) + 86400.0) % 86400.0;
    let h = (s / 3600.0) as i64;
    let m = ((s % 3600.0) / 60.0) as i64;
    let sec = (s % 60.0) as i64;
    let _ = lon_deg;
    format!("{h:02}:{m:02}:{sec:02}")
}

fn main() {
    let site = Site {
        lat_deg: 31.778,
        lon_deg: 35.2354,
        elev_m: 754.0,
    };
    let ref_jd = jd_from_gregorian(2026, 6, 21.0 + 0.5) - site.lon_deg / 360.0;

    for (name, mode) in [
        ("Mishor   ", HorizonMode::Mishor),
        ("Visible  ", HorizonMode::Visible),
        ("Terrain* ", HorizonMode::TerrainProfile),
    ] {
        let optics = Optics {
            horizon_mode: mode,
            limb: LimbReference::Upper,
            horizon_refraction: RefractionModel::Bennett,
            depression_refraction: RefractionModel::None,
        };
        let netz = read_jd(
            &site,
            ref_jd,
            ReadSpec::HorizonCrossing {
                dir: Direction::Rising,
            },
            &optics,
        );
        let shkia = read_jd(
            &site,
            ref_jd,
            ReadSpec::HorizonCrossing {
                dir: Direction::Setting,
            },
            &optics,
        );
        if let (Some(n), Some(s)) = (netz, shkia) {
            let span = s - n;
            let shma = n + 3.0 / 12.0 * span;
            let tefila = n + 4.0 / 12.0 * span;
            let mketana = n + 9.5 / 12.0 * span;
            println!(
                "{name}: netz={} shkia={} | GRA shma={} tefila={} mincha-ketana={}",
                jd_to_local(n, site.lon_deg),
                jd_to_local(s, site.lon_deg),
                jd_to_local(shma, site.lon_deg),
                jd_to_local(tefila, site.lon_deg),
                jd_to_local(mketana, site.lon_deg),
            );
        }
    }
    println!("(*Terrain via scalar read_jd falls back to geometric_dip — NO profile is passed here, matching the proportional-day path in events::bound_jd)");
}
