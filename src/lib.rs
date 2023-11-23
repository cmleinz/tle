#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct InternalDesignator {
    pub last_two_digits_of_launch_year: u8,
    pub launch_number_of_year: u16,
    pub launch_piece: [u8; 3],
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Classification {
    Unclassified,
    Classified,
    Secret,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Tle {
    pub satellite_catalog_number: u32,
    pub classification: Classification,
    pub internal_designator: InternalDesignator,
    pub epoch_year: u8,
    pub epoch_day_of_year: u16,
    pub epoch_fractional_part_of_day: u32,
    pub first_derivative_of_mean_motion: f32,
    pub second_derivative_of_mean_motion: f32,
    pub b_star: f32,
    pub element_set_number: u16,
    pub inclination: f32,
    pub right_ascention_of_ascending_node: f32,
    pub eccentricity: f32,
    pub argument_of_perigee: f32,
    pub mean_anomaly: f32,
    pub mean_motion: f32,
    pub revolution_number_at_epoch: u32,
}
