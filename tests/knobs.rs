//! Halachic-convention knob tests (ADR core-domain/0002/0006/0013): the conventions are
//! parameters and the core honors them. Differential assertions (not oracle values).

use core_engine::events::{read_jd, Bound, Direction, ReadSpec};
use core_engine::optics::{HorizonMode, RefractionModel};
use core_engine::params::Optics;
use core_engine::{time, Site};

fn jeru() -> Site {
    Site {
        lat_deg: 31.78,
        lon_deg: 35.23,
        elev_m: 754.0,
    }
}
fn ref_jd() -> f64 {
    time::jd_from_gregorian(2026, 6, 15.5) // ~local noon anchor
}

/// Mishor (sea-level) vs Visible (elevation dip): at 754 m the visible-horizon netz is several
/// minutes earlier than the mishor netz — the halachic sea-level-vs-visible split (ADR-0013).
#[test]
fn mishor_vs_visible_netz_differ() {
    let netz = ReadSpec::HorizonCrossing {
        dir: Direction::Rising,
    };
    let mishor = Optics {
        horizon_mode: HorizonMode::Mishor,
        ..Optics::default()
    };
    let visible = Optics {
        horizon_mode: HorizonMode::Visible,
        ..Optics::default()
    };
    let a = read_jd(&jeru(), ref_jd(), netz, &mishor).unwrap();
    let b = read_jd(&jeru(), ref_jd(), netz, &visible).unwrap();
    assert!(b < a, "visible netz must precede mishor netz at elevation");
    assert!(
        (a - b) * 86_400.0 > 120.0,
        "754 m dip should shift netz by minutes"
    );
}

/// Refraction is a knob: turning it off moves the horizon crossing.
#[test]
fn refraction_knob_moves_horizon() {
    let netz = ReadSpec::HorizonCrossing {
        dir: Direction::Rising,
    };
    let on = Optics::default(); // Bennett
    let off = Optics {
        horizon_refraction: RefractionModel::None,
        ..Optics::default()
    };
    assert!(read_jd(&jeru(), ref_jd(), netz, &on) != read_jd(&jeru(), ref_jd(), netz, &off));
}

/// GRA vs Magen Avraham via the `proportional_day_bounds` knob (data, not a code branch):
/// MGA's longer day puts sof-zman-shma earlier than GRA's.
#[test]
fn gra_vs_mga_sof_zman_differ() {
    let o = Optics::default();
    let gra = ReadSpec::Proportional {
        fraction: 0.25,
        start: Bound::Netz,
        end: Bound::Shkia,
    };
    let mga = ReadSpec::Proportional {
        fraction: 0.25,
        start: Bound::Depression {
            angle_deg: 16.1,
            dir: Direction::Rising,
        },
        end: Bound::Depression {
            angle_deg: 16.1,
            dir: Direction::Setting,
        },
    };
    let g = read_jd(&jeru(), ref_jd(), gra, &o).unwrap();
    let m = read_jd(&jeru(), ref_jd(), mga, &o).unwrap();
    assert!(m < g, "MGA sof-zman-shma must precede GRA");
}
