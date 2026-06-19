// Cross-engine oracle generator (ADR core-domain/0003/0008, /0017, /0021). Uses KosherJava (an
// INDEPENDENT zmanim engine; LGPL; build/test oracle ONLY — never shipped, never on the correctness
// path) to emit, over a fixed grid: sea-level sunrise/sunset (NOAA calculator, zenith 90.833° =
// geometric sun-center at −0.8333°) AND the literal-72-minute Magen Avraham sof-zman-shma
// (getSofZmanShmaMGA72Minutes — alos/tzais = sunrise∓72 fixed min; the /0021 oracle). Output:
// `name,lat,lon,date,sunrise_millis,sunset_millis,sofzman_mga72_millis`, consumed by tests/cross_engine.rs.
//
// Regenerate (needs the jar; not vendored — LGPL build tool):
//   curl -sSL -o /tmp/kosherjava.jar \
//     https://repo1.maven.org/maven2/com/kosherjava/zmanim/2.5.0/zmanim-2.5.0.jar
//   # full JDK:
//   javac -cp /tmp/kosherjava.jar tools/KosherDiff.java -d /tmp/kd
//   java  -cp /tmp/kosherjava.jar:/tmp/kd KosherDiff > tests/fixtures/kosherjava_vectors.csv
//   # JRE-only (no javac): Java 11+ single-file source launch (JEP 330) compiles in-memory —
//   java -cp /tmp/kosherjava.jar tools/KosherDiff.java > tests/fixtures/kosherjava_vectors.csv

import com.kosherjava.zmanim.ComplexZmanimCalendar;
import com.kosherjava.zmanim.util.GeoLocation;
import java.util.Date;
import java.util.GregorianCalendar;
import java.util.TimeZone;

public class KosherDiff {
    static final double[][] SITES = {
        {31.778, 35.2354}, {40.7128, -74.006}, {0.0, 0.0},
        {-33.8688, 151.2093}, {51.5074, -0.1278}, {25.2048, 55.2708},
    };
    static final String[] NAMES = {"jeru", "nyc", "equator", "sydney", "london", "dubai"};
    static final int[][] DATES = {{2026, 3, 20}, {2026, 6, 21}, {2026, 9, 22}, {2026, 12, 21}};

    public static void main(String[] args) {
        TimeZone utc = TimeZone.getTimeZone("UTC");
        System.out.println("name,lat,lon,date,sunrise_millis,sunset_millis,sofzman_mga72_millis");
        for (int i = 0; i < SITES.length; i++) {
            double lat = SITES[i][0], lon = SITES[i][1];
            // Sea level (elev 0): isolates the solar-position algorithm, no elevation-convention skew.
            GeoLocation gl = new GeoLocation(NAMES[i], lat, lon, 0.0, utc);
            for (int[] d : DATES) {
                ComplexZmanimCalendar ac = new ComplexZmanimCalendar(gl);
                GregorianCalendar cal = new GregorianCalendar(utc);
                cal.clear();
                cal.set(d[0], d[1] - 1, d[2], 12, 0, 0); // month is 0-based
                ac.setCalendar(cal);
                Date sr = ac.getSunrise();
                Date ss = ac.getSunset();
                // Literal-72-min MGA sof-zman-shma: alos72 = sunrise−72, tzais72 = sunset+72 (fixed
                // clock min), shaah = (tzais72−alos72)/12, sof = alos72 + 3·shaah (the /0021 oracle).
                Date sz = ac.getSofZmanShmaMGA72Minutes();
                System.out.printf(
                    "%s,%.4f,%.4f,%04d-%02d-%02d,%s,%s,%s%n",
                    NAMES[i], lat, lon, d[0], d[1], d[2],
                    sr == null ? "null" : Long.toString(sr.getTime()),
                    ss == null ? "null" : Long.toString(ss.getTime()),
                    sz == null ? "null" : Long.toString(sz.getTime()));
            }
        }
    }
}
