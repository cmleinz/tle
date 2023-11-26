#![forbid(unsafe_code)]

const DECIMAL_RADIX: u32 = 10;

/// Some errors are ambiguous as to the line in which they occur.
///
/// Where that is the case, this enum is used to disambiguate.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Line {
    Line1,
    Line2,
}

/// A description of where in the TLE the parsing and validation failed.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Error {
    /// In a TLE each line is required to be 69 characters in length
    ///
    /// For each line, this is first validation conducted
    InvalidLineSize(Line, usize),
    /// Certain segments of the TLE are required to be an ASCII space character. In those cases,
    /// if another character is encountered this error will be produced with the character found,
    /// and the position found.
    Space(Line, char, usize),
    /// Failed to parse the satellite catalog number in the given line
    SatlliteCatalogNumber(Line),
    /// The classification must be one of three characters
    ///
    /// 1. 'U': Unclassified
    /// 1. 'C': Classified
    /// 1. 'S': Secret
    ///
    /// If the classification field matches none of these, this error will be produced
    Classification(char),
    /// Represents a failure to parse the international designator's two digit launch year
    InternationalDesignatorLaunchYear,
    /// Represents a failure to parse the international designator's three digit launch number
    InternationalDesignatorLaunchNumber,
    /// Represents a failure to parse the two digit epoch year
    EpochYear,
    /// Represents a failure to parse the epoch day
    EpochDay,
    /// Represents a failure to parse the first derivative of mean motion
    FirstDerivative,
    /// Represents a failure to parse the second derivative of mean motion
    SecondDerivative,
    /// Represents a failure to parse B* drag term
    BStar,
    /// Represents a validation failure, the ephemeris type must be '0'
    EphemerisType(char),
    /// Represents a failure to parse the element set number
    ElementSetNumber,
    /// Represents a failure to parse the inclination
    Inclination,
    /// Represents a failure to parse the right ascension
    RightAscension,
    /// Represents a failure to parse the eccentricity
    Eccentricty,
    /// Represents a failure to parse the argument of perigee
    ArgumentOfPerigee,
    /// Represents a failure to parse the mean anomaly
    MeanAnomaly,
    /// Represents a failure to parse the mean motion
    MeanMotion,
    /// Represents a failure to parse the revolution number
    RevolutionNumber,
    /// Represents a failure to parse the checksum for the given line.
    ///
    /// This value must be a modulo 10 number
    Checksum(Line, char),
    /// The TLE checksum is calculated by summing all of the digits in the line, plus 1 for each '-'
    /// character, modulo 10.
    ///
    /// If the calculated checksum does not match the one provided in the line, the line number,
    /// found checksum, and calculated checksum will be returned
    InvalidChecksum(Line, u8, u8),
    /// The first character of each line must be the number of the line i.e. '1' and '2'.
    ///
    /// If that check fails this error is propagated with the character found, and the line which
    /// failed.
    LineNumber(Line, char),
    /// The second sequence in each line of the TLE is the satellite catalog number, this must be
    /// consistent across both lines
    ///
    /// In the event of this error the values represent the satellite catalog numbers found in each
    /// line
    SatelliteCatalogNumberMismatch(u32, u32),
    /// TLEs are required to contain only valid ASCII characters
    ContainsNonAsciiCharacter(Line),
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InternationalDesignator {
    pub launch_year: u8,
    pub launch_num: u16,
    pub launch_piece: [char; 3],
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Classification {
    Unclassified,
    Classified,
    Secret,
}

/// A parsed and validate Two Line Element Set
///
/// This is primarily generated via the `parse` method
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Tle {
    pub satellite_catalog_number: u32,
    pub classification: Classification,
    pub international_designator: InternationalDesignator,
    pub epoch_year: u8,
    pub epoch_day_and_fractional_part: f64,
    pub first_derivative_of_mean_motion: f32,
    pub second_derivative_of_mean_motion: f32,
    pub b_star: f32,
    pub element_set_number: u16,
    checksum_1: u8,
    pub inclination: f32,
    pub right_ascension_of_ascending_node: f32,
    pub eccentricity: f32,
    pub argument_of_perigee: f32,
    pub mean_anomaly: f32,
    pub mean_motion: f32,
    pub revolution_number_at_epoch: u32,
    checksum_2: u8,
}

macro_rules! split_space {
    ($line_num:expr, $line:ident, $pos:literal) => {{
        let (slice, line) = $line.split_at(1);
        if slice[0] != ' ' {
            return Err(Error::Space($line_num, slice[0], $pos));
        }

        line
    }};
}

impl Tle {
    const LINE_LEN: usize = 69;

    pub fn parse(line1: &[u8], line2: &[u8]) -> Result<Self, Error> {
        let line = match validate_line(line1, Line::Line1) {
            Ok(l) => l,
            Err(error) => return Err(error),
        };

        let (slice, line) = line.split_at(1);
        if slice[0] != '1' {
            return Err(Error::LineNumber(Line::Line1, line[0]));
        }

        let line = split_space!(Line::Line1, line, 1);

        let (slice, line) = line.split_at(5);
        let Some(satellite_catalog_number_1) = as_digits(slice) else {
            return Err(Error::SatlliteCatalogNumber(Line::Line1));
        };

        let (slice, line) = line.split_at(1);
        let classification = match slice[0] {
            'U' => Classification::Unclassified,
            'C' => Classification::Classified,
            'S' => Classification::Secret,
            found => return Err(Error::Classification(found)),
        };

        let line = split_space!(Line::Line1, line, 8);

        let (slice, line) = line.split_at(2);
        let Some(launch_year) = as_digits(slice) else {
            return Err(Error::InternationalDesignatorLaunchYear);
        };

        let (slice, line) = line.split_at(3);
        let Some(launch_num) = as_digits(slice) else {
            return Err(Error::InternationalDesignatorLaunchNumber);
        };

        let (slice, line) = line.split_at(3);
        let launch_piece = [slice[0], slice[1], slice[2]];
        let internal_designator = InternationalDesignator {
            launch_year: launch_year as u8,
            launch_num: launch_num as u16,
            launch_piece,
        };

        let line = split_space!(Line::Line1, line, 17);

        let (slice, line) = line.split_at(2);
        let Some(epoch_year) = as_digits(slice) else {
            return Err(Error::EpochYear);
        };
        let (slice, line) = line.split_at(12);
        // TODO: Make this const evalable and remove allocations
        let Ok(epoch_day_and_fractional_part) = String::from_iter(slice).parse::<f64>() else {
            return Err(Error::EpochDay);
        };

        let line = split_space!(Line::Line1, line, 32);

        let (slice, line) = line.split_at(10);
        let slice = trim_leading_space(slice);
        let Ok(first_derivative_of_mean_motion) = String::from_iter(slice).parse::<f32>() else {
            return Err(Error::FirstDerivative);
        };

        let line = split_space!(Line::Line1, line, 43);

        let (slice, line) = line.split_at(8);
        let second_derivative_of_mean_motion = match parse_tle_f32(slice) {
            Ok(s) => s,
            Err(e) => return Err(e),
        };

        let line = split_space!(Line::Line1, line, 52);

        let (mut slice, line) = line.split_at(8);
        let mut sign = 1.0;
        if slice[0] == '-' {
            sign = -1.0;
            let (_neg_sign, s) = slice.split_first().unwrap();
            slice = s;
        }

        let b_star = match parse_tle_f32(slice) {
            Ok(s) => s * sign,
            Err(e) => return Err(e),
        };

        let line = split_space!(Line::Line1, line, 61);
        let (slice, line) = line.split_at(1);
        if slice[0] != '0' {
            return Err(Error::EphemerisType(slice[0]));
        }

        let line = split_space!(Line::Line1, line, 63);

        let (slice, line) = line.split_at(4);
        let slice = trim_leading_space(slice);

        let Some(element_set_number) = as_digits(slice) else {
            return Err(Error::ElementSetNumber);
        };

        let Some(checksum_1) = as_digits(line) else {
            return Err(Error::Checksum(Line::Line1, line[0]));
        };
        let checksum_1 = checksum_1 as u8;

        let line = match validate_line(line2, Line::Line2) {
            Ok(l) => l,
            Err(error) => return Err(error),
        };

        let (slice, line) = line.split_first().unwrap();
        if *slice != '2' {
            return Err(Error::LineNumber(Line::Line2, *slice));
        }

        let line = split_space!(Line::Line1, line, 1);

        let (slice, line) = line.split_at(5);
        let Some(satellite_catalog_number_2) = as_digits(slice) else {
            return Err(Error::SatlliteCatalogNumber(Line::Line2));
        };

        if satellite_catalog_number_1 != satellite_catalog_number_2 {
            return Err(Error::SatelliteCatalogNumberMismatch(
                satellite_catalog_number_1,
                satellite_catalog_number_2,
            ));
        }
        let satellite_catalog_number = satellite_catalog_number_1;

        let line = split_space!(Line::Line2, line, 7);

        let (slice, line) = line.split_at(8);
        let slice = trim_leading_space(slice);
        let Ok(inclination) = String::from_iter(slice).parse::<f32>() else {
            return Err(Error::Inclination);
        };

        let line = split_space!(Line::Line2, line, 16);

        let (slice, line) = line.split_at(8);
        let Ok(right_ascension_of_ascending_node) = String::from_iter(slice).parse::<f32>() else {
            return Err(Error::RightAscension);
        };

        let line = split_space!(Line::Line2, line, 25);

        let (slice, line) = line.split_at(7);
        let Some(eccentricity) = as_digits(slice) else {
            return Err(Error::Eccentricty);
        };
        let Some(dig) = eccentricity.checked_ilog10() else {
            return Err(Error::Eccentricty);
        };
        let leading_zeroes = dig as i32 - slice.len() as i32;
        let eccentricity = (eccentricity as f32).powi(leading_zeroes);

        let line = split_space!(Line::Line2, line, 33);

        let (slice, line) = line.split_at(8);
        let Ok(argument_of_perigee) = String::from_iter(slice).parse::<f32>() else {
            return Err(Error::ArgumentOfPerigee);
        };

        let line = split_space!(Line::Line2, line, 42);

        let (slice, line) = line.split_at(8);
        let Ok(mean_anomaly) = String::from_iter(slice).parse::<f32>() else {
            return Err(Error::MeanAnomaly);
        };

        let line = split_space!(Line::Line2, line, 51);

        let (slice, line) = line.split_at(11);
        let Ok(mean_motion) = String::from_iter(slice).parse::<f32>() else {
            return Err(Error::MeanAnomaly);
        };

        let (slice, line) = line.split_at(5);
        let Some(revolution_number_at_epoch) = as_digits(slice) else {
            return Err(Error::MeanMotion);
        };

        let Some(checksum_2) = as_digits(line) else {
            return Err(Error::Checksum(Line::Line2, line[0]));
        };
        let checksum_2 = checksum_2 as u8;

        let me = Tle {
            satellite_catalog_number,
            classification,
            international_designator: internal_designator,
            epoch_year: epoch_year as u8,
            epoch_day_and_fractional_part,
            first_derivative_of_mean_motion,
            second_derivative_of_mean_motion,
            b_star,
            element_set_number: element_set_number as u16,
            checksum_1,
            inclination,
            right_ascension_of_ascending_node,
            eccentricity,
            argument_of_perigee,
            mean_anomaly,
            mean_motion,
            revolution_number_at_epoch,
            checksum_2,
        };

        Ok(me)
    }
}

const fn trim_leading_space(line: &[char]) -> &[char] {
    let mut blank = 0;
    while blank <= line.len() {
        if line[blank] == ' ' {
            blank += 1;
        } else {
            break;
        }
    }
    let (_trimmmed, slice) = line.split_at(blank);
    slice
}

fn parse_tle_f32(line: &[char]) -> Result<f32, Error> {
    let trimmed = trim_leading_space(line);

    let mut idx = None;
    let mut i = 0;
    while i < trimmed.len() {
        if trimmed[i] == '-' {
            if idx.is_none() {
                idx = Some(i);
            } else {
                return Err(Error::SecondDerivative);
            }
        }
        i += 1;
    }

    let Some(idx) = idx else {
        return Err(Error::SecondDerivative);
    };
    let (num, exp) = trimmed.split_at(idx);
    let Some(num) = as_digits(num) else {
        return Err(Error::SecondDerivative);
    };
    let Some((neg, exp)) = exp.split_first() else {
        return Err(Error::SecondDerivative);
    };
    assert_eq!(*neg, '-');
    let Some(exp) = as_digits(exp) else {
        return Err(Error::SecondDerivative);
    };

    let val = (num as f32).powi(-(exp as i32));

    Ok(val)
}

const fn validate_line(line: &[u8], line_num: Line) -> Result<[char; Tle::LINE_LEN], Error> {
    if line.len() != Tle::LINE_LEN {
        return Err(Error::InvalidLineSize(line_num, line.len()));
    }

    if !line.is_ascii() {
        return Err(Error::ContainsNonAsciiCharacter(line_num));
    }

    Ok(tle_line(line))
}

/// # Panics
///
/// Panics if the line is less than the Tle::LINE_LEN
const fn tle_line(line: &[u8]) -> [char; Tle::LINE_LEN] {
    let mut arr = [char::MAX; Tle::LINE_LEN];
    let mut i = 0;
    while i < Tle::LINE_LEN {
        arr[i] = line[i] as char;
        i += 1;
    }
    arr
}

const fn as_digits(chars: &[char]) -> Option<u32> {
    if chars.len() == 0 {
        return None;
    }

    let mut val: u32 = 0;

    let mut i = 0;

    while i < chars.len() {
        let Some(ascii_val) = chars[chars.len() - 1 - i].to_digit(DECIMAL_RADIX) else {
            return None;
        };
        val += ascii_val * 10u32.pow(i as u32);
        i += 1;
    }

    Some(val)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tle_test() {
        // ISS
        let line1 = b"1 25544U 98067A   08264.51782528 -.00002182  00000-0 -11606-4 0  2927";
        let line2 = b"2 25544  51.6416 247.4627 0006703 130.5360 325.0288 15.72125391563537";
        let _ = Tle::parse(line1, line2).unwrap();
        // NOAA 14
        let line1 = b"1 23455U 94089A   97320.90946019  .00000140  00000-0  10191-3 0  2621";
        let line2 = b"2 23455  99.0090 272.6745 0008546 223.1686 136.8816 14.11711747148495";
        let _ = Tle::parse(line1, line2).unwrap();
    }

    #[test]
    fn as_digits_is_valid() {
        let x = ['1', '2', '3', '4'];
        assert_eq!(as_digits(&x), Some(1234_u32));
        let x = ['0', '2', '3', '4'];
        assert_eq!(as_digits(&x), Some(234_u32));
        let x = ['9', '0', '0', '9'];
        assert_eq!(as_digits(&x), Some(9009_u32));
        let x = ['8'];
        assert_eq!(as_digits(&x), Some(8_u32));
    }
}
