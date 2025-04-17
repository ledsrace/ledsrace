use core::{cell::Cell, f32};
use embassy_time::Duration;
use heapless::Vec;
use libm::{fabsf, sinf};

use super::{Animation, Color, LedStateBuffer, Priority};

/// Represents a point on the circuit
#[derive(Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

impl Point {
    const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn distance_to(&self, other: &Point) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        libm::sqrtf(dx * dx + dy * dy)
    }
}

/// Static array of LED positions on the circuit
pub const LED_POSITIONS_RANDOM: [Point; 216] = [
    Point::new(40.43, 108.65),
    Point::new(85.79, 90.52),
    Point::new(36.71, 99.86),
    Point::new(169.91, 83.07),
    Point::new(140.89, 117.0),
    Point::new(106.89, 97.79),
    Point::new(60.32, 103.17),
    Point::new(155.93, 88.18),
    Point::new(115.93, 99.23),
    Point::new(97.8, 95.19),
    Point::new(49.57, 111.03),
    Point::new(58.51, 71.75),
    Point::new(73.84, 153.76),
    Point::new(159.66, 62.2),
    Point::new(31.6, 87.81),
    Point::new(133.89, 99.79),
    Point::new(150.95, 84.2),
    Point::new(65.88, 88.86),
    Point::new(147.91, 83.02),
    Point::new(156.76, 94.7),
    Point::new(127.89, 99.79),
    Point::new(145.12, 59.61),
    Point::new(96.23, 106.63),
    Point::new(112.87, 98.89),
    Point::new(63.89, 117.79),
    Point::new(29.9, 38.09),
    Point::new(30.3, 84.77),
    Point::new(42.8, 114.29),
    Point::new(163.98, 71.15),
    Point::new(74.0, 84.69),
    Point::new(60.69, 59.81),
    Point::new(128.65, 67.4),
    Point::new(128.8, 113.31),
    Point::new(77.19, 86.0),
    Point::new(109.84, 98.38),
    Point::new(150.94, 59.14),
    Point::new(53.09, 138.6),
    Point::new(57.0, 83.89),
    Point::new(23.0, 66.81),
    Point::new(134.87, 115.91),
    Point::new(83.1, 89.09),
    Point::new(27.14, 40.13),
    Point::new(56.6, 147.67),
    Point::new(24.77, 42.69),
    Point::new(75.07, 107.85),
    Point::new(66.62, 129.82),
    Point::new(70.44, 144.91),
    Point::new(87.01, 108.19),
    Point::new(48.5, 104.68),
    Point::new(54.3, 141.6),
    Point::new(34.19, 93.89),
    Point::new(139.89, 99.79),
    Point::new(166.75, 77.25),
    Point::new(147.97, 59.34),
    Point::new(51.8, 135.41),
    Point::new(116.95, 106.47),
    Point::new(39.07, 34.45),
    Point::new(62.1, 50.76),
    Point::new(48.3, 33.89),
    Point::new(113.98, 105.26),
    Point::new(131.76, 114.79),
    Point::new(135.61, 80.89),
    Point::new(57.1, 34.69),
    Point::new(75.67, 165.76),
    Point::new(59.77, 65.83),
    Point::new(148.89, 99.79),
    Point::new(66.7, 123.97),
    Point::new(51.09, 34.0),
    Point::new(107.89, 104.79),
    Point::new(67.56, 135.82),
    Point::new(21.19, 51.69),
    Point::new(175.7, 91.92),
    Point::new(145.89, 99.79),
    Point::new(63.24, 104.25),
    Point::new(119.89, 107.79),
    Point::new(66.89, 132.79),
    Point::new(125.92, 111.6),
    Point::new(57.64, 77.83),
    Point::new(62.8, 41.77),
    Point::new(41.6, 111.61),
    Point::new(99.05, 105.91),
    Point::new(142.07, 60.25),
    Point::new(32.9, 90.89),
    Point::new(69.7, 168.95),
    Point::new(29.01, 81.86),
    Point::new(72.1, 107.17),
    Point::new(57.19, 102.24),
    Point::new(156.99, 91.4),
    Point::new(88.62, 91.87),
    Point::new(45.3, 120.39),
    Point::new(65.89, 120.79),
    Point::new(80.19, 87.5),
    Point::new(142.89, 99.79),
    Point::new(66.62, 126.95),
    Point::new(54.88, 113.85),
    Point::new(55.43, 144.66),
    Point::new(129.36, 79.01),
    Point::new(130.97, 65.69),
    Point::new(26.59, 75.89),
    Point::new(75.03, 156.75),
    Point::new(66.27, 168.14),
    Point::new(70.75, 84.74),
    Point::new(168.35, 80.25),
    Point::new(25.3, 72.89),
    Point::new(61.18, 56.84),
    Point::new(121.89, 99.79),
    Point::new(127.13, 76.43),
    Point::new(52.0, 112.72),
    Point::new(104.89, 104.91),
    Point::new(62.39, 47.78),
    Point::new(136.89, 99.79),
    Point::new(45.19, 33.89),
    Point::new(126.61, 69.86),
    Point::new(75.69, 159.79),
    Point::new(62.64, 44.82),
    Point::new(110.89, 104.79),
    Point::new(50.79, 102.13),
    Point::new(22.09, 63.69),
    Point::new(78.05, 108.17),
    Point::new(44.0, 117.29),
    Point::new(149.89, 117.23),
    Point::new(68.44, 138.86),
    Point::new(130.89, 99.79),
    Point::new(71.38, 147.89),
    Point::new(68.1, 86.52),
    Point::new(72.5, 150.92),
    Point::new(57.26, 80.88),
    Point::new(101.98, 105.29),
    Point::new(35.41, 96.79),
    Point::new(83.97, 108.22),
    Point::new(57.7, 150.57),
    Point::new(69.2, 106.29),
    Point::new(146.89, 117.23),
    Point::new(178.46, 97.81),
    Point::new(154.02, 59.32),
    Point::new(178.89, 100.79),
    Point::new(47.91, 126.46),
    Point::new(157.1, 60.22),
    Point::new(153.89, 85.8),
    Point::new(62.69, 90.09),
    Point::new(136.32, 62.63),
    Point::new(60.81, 115.98),
    Point::new(21.47, 60.68),
    Point::new(60.19, 62.8),
    Point::new(66.22, 105.35),
    Point::new(139.07, 61.34),
    Point::new(94.75, 94.13),
    Point::new(126.09, 73.27),
    Point::new(133.63, 64.17),
    Point::new(175.7, 109.29),
    Point::new(161.93, 116.61),
    Point::new(76.0, 162.76),
    Point::new(69.44, 141.88),
    Point::new(173.58, 89.0),
    Point::new(170.67, 113.21),
    Point::new(152.89, 117.23),
    Point::new(143.89, 117.1),
    Point::new(159.0, 117.0),
    Point::new(165.27, 74.27),
    Point::new(60.91, 159.69),
    Point::new(155.14, 97.72),
    Point::new(155.95, 117.25),
    Point::new(138.67, 81.19),
    Point::new(59.19, 68.88),
    Point::new(60.1, 36.09),
    Point::new(141.75, 81.52),
    Point::new(91.57, 93.08),
    Point::new(21.08, 57.77),
    Point::new(39.2, 105.81),
    Point::new(22.9, 45.59),
    Point::new(178.89, 103.69),
    Point::new(48.12, 108.0),
    Point::new(124.89, 99.79),
    Point::new(167.89, 114.69),
    Point::new(73.13, 168.36),
    Point::new(103.76, 96.92),
    Point::new(62.12, 162.74),
    Point::new(57.5, 86.88),
    Point::new(161.24, 65.05),
    Point::new(152.1, 99.19),
    Point::new(137.88, 116.7),
    Point::new(49.19, 129.5),
    Point::new(59.89, 156.79),
    Point::new(42.09, 34.0),
    Point::new(27.8, 78.89),
    Point::new(132.52, 80.43),
    Point::new(63.6, 165.73),
    Point::new(36.09, 35.19),
    Point::new(93.33, 107.42),
    Point::new(38.01, 102.82),
    Point::new(57.88, 114.94),
    Point::new(50.5, 132.5),
    Point::new(122.93, 109.66),
    Point::new(165.02, 115.85),
    Point::new(21.03, 54.65),
    Point::new(54.17, 101.58),
    Point::new(58.8, 153.64),
    Point::new(62.3, 38.59),
    Point::new(173.3, 111.39),
    Point::new(61.69, 53.8),
    Point::new(21.8, 48.71),
    Point::new(54.1, 34.13),
    Point::new(90.3, 108.0),
    Point::new(118.94, 99.6),
    Point::new(24.09, 69.81),
    Point::new(177.48, 94.85),
    Point::new(32.93, 36.51),
    Point::new(144.86, 82.09),
    Point::new(58.02, 74.79),
    Point::new(80.94, 108.19),
    Point::new(46.69, 123.39),
    Point::new(162.62, 68.05),
    Point::new(59.59, 89.3),
    Point::new(100.76, 96.1),
    Point::new(177.89, 106.79),
    Point::new(171.73, 86.22),
];

/// LED positions in sorted order, following the physical layout of the circuit
pub const LED_POSITIONS_SORTED: [Point; 216] = [
    Point::new(44.00, 117.30),  // 1
    Point::new(45.30, 120.40),  // 2
    Point::new(46.70, 123.40),  // 3
    Point::new(47.91, 126.47),  // 4
    Point::new(49.20, 129.50),  // 5
    Point::new(50.50, 132.50),  // 6
    Point::new(51.81, 135.42),  // 7
    Point::new(53.10, 138.60),  // 8
    Point::new(54.30, 141.60),  // 9
    Point::new(55.44, 144.67),  // 10
    Point::new(56.60, 147.68),  // 11
    Point::new(57.71, 150.57),  // 12
    Point::new(58.81, 153.65),  // 13
    Point::new(59.90, 156.80),  // 14
    Point::new(60.91, 159.70),  // 15
    Point::new(62.12, 162.74),  // 16
    Point::new(63.60, 165.74),  // 17
    Point::new(66.28, 168.14),  // 18
    Point::new(69.71, 168.96),  // 19
    Point::new(73.14, 168.37),  // 20
    Point::new(75.67, 165.76),  // 21
    Point::new(76.01, 162.76),  // 22
    Point::new(75.70, 159.80),  // 23
    Point::new(75.03, 156.76),  // 24
    Point::new(73.85, 153.77),  // 25
    Point::new(72.51, 150.92),  // 26
    Point::new(71.38, 147.90),  // 27
    Point::new(70.45, 144.92),  // 28
    Point::new(69.44, 141.89),  // 29
    Point::new(68.45, 138.87),  // 30
    Point::new(67.57, 135.83),  // 31
    Point::new(66.90, 132.80),  // 32
    Point::new(66.63, 129.82),  // 33
    Point::new(66.62, 126.96),  // 34
    Point::new(66.71, 123.97),  // 35
    Point::new(65.90, 120.80),  // 36
    Point::new(63.90, 117.80),  // 37
    Point::new(60.81, 115.98),  // 38
    Point::new(57.88, 114.94),  // 39
    Point::new(54.89, 113.85),  // 40
    Point::new(52.01, 112.72),  // 41
    Point::new(49.58, 111.04),  // 42
    Point::new(48.13, 108.00),  // 43
    Point::new(48.51, 104.68),  // 44
    Point::new(50.79, 102.13),  // 45
    Point::new(54.18, 101.58),  // 46
    Point::new(57.20, 102.24),  // 47
    Point::new(60.33, 103.17),  // 48
    Point::new(63.25, 104.25),  // 49
    Point::new(66.23, 105.35),  // 50
    Point::new(69.20, 106.30),  // 51
    Point::new(72.10, 107.18),  // 52
    Point::new(75.08, 107.85),  // 53
    Point::new(78.05, 108.18),  // 54
    Point::new(80.94, 108.19),  // 55
    Point::new(83.98, 108.22),  // 56
    Point::new(87.02, 108.20),  // 57
    Point::new(90.30, 108.00),  // 58
    Point::new(93.34, 107.43),  // 59
    Point::new(96.24, 106.64),  // 60
    Point::new(99.05, 105.91),  // 61
    Point::new(101.98, 105.29), // 62
    Point::new(104.90, 104.91), // 63
    Point::new(107.90, 104.80), // 64
    Point::new(110.90, 104.80), // 65
    Point::new(113.98, 105.27), // 66
    Point::new(116.96, 106.48), // 67
    Point::new(119.90, 107.80), // 68
    Point::new(122.94, 109.67), // 69
    Point::new(125.92, 111.60), // 70
    Point::new(128.80, 113.31), // 71
    Point::new(131.77, 114.79), // 72
    Point::new(134.87, 115.91), // 73
    Point::new(137.88, 116.71), // 74
    Point::new(140.90, 117.00), // 75
    Point::new(143.90, 117.10), // 76
    Point::new(146.90, 117.24), // 77
    Point::new(149.90, 117.24), // 78
    Point::new(152.90, 117.24), // 79
    Point::new(155.96, 117.25), // 80
    Point::new(159.00, 117.00), // 81
    Point::new(161.94, 116.61), // 82
    Point::new(165.03, 115.85), // 83
    Point::new(167.90, 114.70), // 84
    Point::new(170.68, 113.22), // 85
    Point::new(173.30, 111.40), // 86
    Point::new(175.71, 109.29), // 87
    Point::new(177.90, 106.80), // 88
    Point::new(178.90, 103.70), // 89
    Point::new(178.90, 100.80), // 90
    Point::new(178.47, 97.81),  // 91
    Point::new(177.48, 94.85),  // 92
    Point::new(175.70, 91.92),  // 93
    Point::new(173.58, 89.00),  // 94
    Point::new(171.73, 86.23),  // 95
    Point::new(169.91, 83.07),  // 96
    Point::new(168.35, 80.25),  // 97
    Point::new(166.75, 77.26),  // 98
    Point::new(165.28, 74.28),  // 99
    Point::new(163.98, 71.16),  // 100
    Point::new(162.62, 68.05),  // 101
    Point::new(161.24, 65.05),  // 102
    Point::new(159.66, 62.21),  // 103
    Point::new(157.10, 60.23),  // 104
    Point::new(154.03, 59.33),  // 105
    Point::new(150.95, 59.15),  // 106
    Point::new(147.97, 59.35),  // 107
    Point::new(145.12, 59.61),  // 108
    Point::new(142.08, 60.25),  // 109
    Point::new(139.08, 61.35),  // 110
    Point::new(136.32, 62.64),  // 111
    Point::new(133.64, 64.17),  // 112
    Point::new(130.98, 65.70),  // 113
    Point::new(128.66, 67.41),  // 114
    Point::new(126.61, 69.86),  // 115
    Point::new(126.10, 73.28),  // 116
    Point::new(127.13, 76.43),  // 117
    Point::new(129.36, 79.02),  // 118
    Point::new(132.53, 80.43),  // 119
    Point::new(135.61, 80.90),  // 120
    Point::new(138.67, 81.19),  // 121
    Point::new(141.75, 81.53),  // 122
    Point::new(144.86, 82.10),  // 123
    Point::new(147.91, 83.03),  // 124
    Point::new(150.95, 84.20),  // 125
    Point::new(153.90, 85.80),  // 126
    Point::new(155.93, 88.18),  // 127
    Point::new(156.99, 91.41),  // 128
    Point::new(156.77, 94.71),  // 129
    Point::new(155.15, 97.73),  // 130
    Point::new(152.10, 99.19),  // 131
    Point::new(148.90, 99.80),  // 132
    Point::new(145.90, 99.80),  // 133
    Point::new(142.90, 99.80),  // 134
    Point::new(139.90, 99.80),  // 135
    Point::new(136.90, 99.80),  // 136
    Point::new(133.90, 99.80),  // 137
    Point::new(130.90, 99.80),  // 138
    Point::new(127.90, 99.80),  // 139
    Point::new(124.90, 99.80),  // 140
    Point::new(121.90, 99.80),  // 141
    Point::new(118.95, 99.60),  // 142
    Point::new(115.93, 99.24),  // 143
    Point::new(112.87, 98.89),  // 144
    Point::new(109.85, 98.38),  // 145
    Point::new(106.90, 97.80),  // 146
    Point::new(103.77, 96.93),  // 147
    Point::new(100.76, 96.10),  // 148
    Point::new(97.81, 95.20),   // 149
    Point::new(94.75, 94.14),   // 150
    Point::new(91.58, 93.09),   // 151
    Point::new(88.62, 91.88),   // 152
    Point::new(85.80, 90.52),   // 153
    Point::new(83.10, 89.10),   // 154
    Point::new(80.20, 87.50),   // 155
    Point::new(77.20, 86.00),   // 156
    Point::new(74.00, 84.70),   // 157
    Point::new(70.76, 84.74),   // 158
    Point::new(68.10, 86.53),   // 159
    Point::new(65.89, 88.86),   // 160
    Point::new(62.70, 90.10),   // 161
    Point::new(59.59, 89.30),   // 162
    Point::new(57.51, 86.88),   // 163
    Point::new(57.00, 83.90),   // 164
    Point::new(57.26, 80.88),   // 165
    Point::new(57.64, 77.83),   // 166
    Point::new(58.02, 74.79),   // 167
    Point::new(58.51, 71.75),   // 168
    Point::new(59.19, 68.88),   // 169
    Point::new(59.78, 65.84),   // 170
    Point::new(60.20, 62.80),   // 171
    Point::new(60.70, 59.82),   // 172
    Point::new(61.18, 56.85),   // 173
    Point::new(61.70, 53.80),   // 174
    Point::new(62.10, 50.77),   // 175
    Point::new(62.40, 47.79),   // 176
    Point::new(62.64, 44.83),   // 177
    Point::new(62.80, 41.78),   // 178
    Point::new(62.30, 38.60),   // 179
    Point::new(60.10, 36.10),   // 180
    Point::new(57.10, 34.70),   // 181
    Point::new(54.10, 34.14),   // 182
    Point::new(51.10, 34.00),   // 183
    Point::new(48.30, 33.90),   // 184
    Point::new(45.20, 33.90),   // 185
    Point::new(42.10, 34.00),   // 186
    Point::new(39.07, 34.46),   // 187
    Point::new(36.10, 35.19),   // 188
    Point::new(32.93, 36.51),   // 189
    Point::new(29.90, 38.10),   // 190
    Point::new(27.15, 40.14),   // 191
    Point::new(24.77, 42.70),   // 192
    Point::new(22.90, 45.60),   // 193
    Point::new(21.81, 48.72),   // 194
    Point::new(21.20, 51.70),   // 195
    Point::new(21.04, 54.66),   // 196
    Point::new(21.09, 57.78),   // 197
    Point::new(21.47, 60.68),   // 198
    Point::new(22.10, 63.70),   // 199
    Point::new(23.00, 66.81),   // 200
    Point::new(24.10, 69.82),   // 201
    Point::new(25.30, 72.90),   // 202
    Point::new(26.60, 75.90),   // 203
    Point::new(27.80, 78.90),   // 204
    Point::new(29.01, 81.87),   // 205
    Point::new(30.30, 84.78),   // 206
    Point::new(31.61, 87.82),   // 207
    Point::new(32.90, 90.90),   // 208
    Point::new(34.20, 93.90),   // 209
    Point::new(35.41, 96.80),   // 210
    Point::new(36.71, 99.87),   // 211
    Point::new(38.01, 102.82),  // 212
    Point::new(39.20, 105.81),  // 213
    Point::new(40.44, 108.66),  // 214
    Point::new(41.61, 111.62),  // 215
    Point::new(42.80, 114.30),  // 216
];

/// Calculate the geometric center of the circuit
pub fn calculate_center(led_positions: &[Point]) -> Point {
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;

    for point in led_positions.iter() {
        sum_x += point.x;
        sum_y += point.y;
    }

    Point::new(
        sum_x / led_positions.len() as f32,
        sum_y / led_positions.len() as f32,
    )
}

pub fn calculate_center_middle(led_positions: &[Point]) -> Point {
    // find minimum and maximum for x and y.
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for point in led_positions.iter() {
        min_x = min_x.min(point.x);
        min_y = min_y.min(point.y);
        max_x = max_x.max(point.x);
        max_y = max_y.max(point.y);
    }

    Point::new((min_x + max_x) / 2.0, (min_y + max_y) / 2.0)
}

/// Get the maximum distance from the center to any LED
pub fn max_distance_from_center(led_positions: &[Point]) -> f32 {
    let center = calculate_center(led_positions);
    let mut max_distance = 0.0;

    for point in led_positions.iter() {
        let distance = point.distance_to(&center);
        if distance > max_distance {
            max_distance = distance;
        }
    }

    max_distance
}

/// Animation that creates pulses moving outward from the center
pub struct CircuitPulse {
    /// How fast the pulse moves (units per second)
    pub speed: f32,
    /// Width of the pulse (in distance units)
    pub pulse_width: f32,
    /// Colors of the pulse
    // pub colors: heapless::Vec<Color, 3>,
    // pub color: Color,
    /// Time between pulses (None for single pulse)
    pub repeat_interval: Option<Duration>,
    /// Time when the animation started
    start_time: Cell<Option<Duration>>,

    /// Whether to randomize the position of the pulse
    randomize: bool,
}

unsafe impl Sync for CircuitPulse {}

impl CircuitPulse {
    pub const fn new(
        speed: f32,
        pulse_width: f32,
        // colors: heapless::Vec<Color, 3>,
        repeat_interval: Option<Duration>,
        randomize: bool,
    ) -> Self {
        Self {
            speed,
            pulse_width,
            // colors,
            repeat_interval,
            start_time: Cell::new(None),
            randomize,
        }
    }

    fn calculate_brightness(&self, distance: f32, pulse_distance: f32) -> u8 {
        // Distance from the pulse center
        let delta = fabsf(distance - pulse_distance);

        // If we're within the pulse width, calculate brightness
        if delta <= self.pulse_width {
            // Create a smooth falloff at the edges of the pulse
            let normalized = 1.0 - (delta / self.pulse_width);
            let brightness = normalized * normalized; // Square for smoother falloff
            (brightness * 150.0) as u8 // Max brightness of 150
        } else {
            0
        }
    }
}

impl<const N: usize> Animation<N> for CircuitPulse {
    fn reset(&self) {
        self.start_time.set(None);
    }

    fn render(&self, timestamp: Duration, buffer: &mut LedStateBuffer<N>) {
        let led_positions = if self.randomize {
            &LED_POSITIONS_RANDOM
        } else {
            &LED_POSITIONS_SORTED
        };

        // Initialize start time if not set
        if self.start_time.get().is_none() {
            self.start_time.set(Some(timestamp));
        }

        let elapsed = timestamp - self.start_time.get().unwrap();
        let center = calculate_center(led_positions);
        let max_distance = max_distance_from_center(led_positions);

        // Calculate how far the pulse has traveled
        let pulse_distance =
            (elapsed.as_micros() as f32 * 1e-6 * self.speed) % (max_distance * 2.0);

        let colors: Vec<Color, 3> = Vec::from_iter([
            Color(150, 0, 100), // Purple color
            Color(150, 50, 50), // Purple color
            Color(150, 10, 10), // Purple color
        ]);

        let num_colors = colors.len();

        // For each LED, calculate its brightness based on distance from pulse
        for (i, pos) in led_positions.iter().enumerate() {
            let distance = pos.distance_to(&center);
            let brightness = self.calculate_brightness(distance, pulse_distance);

            let color_index = (i % num_colors) as usize;

            if brightness > 0 {
                buffer.set_led(
                    i,
                    Color(
                        (colors[color_index].0 as u32 * brightness as u32 / 150) as u8,
                        (colors[color_index].1 as u32 * brightness as u32 / 150) as u8,
                        (colors[color_index].2 as u32 * brightness as u32 / 150) as u8,
                    ),
                    Priority::Normal,
                );
            }
        }
    }

    fn is_finished(&self) -> bool {
        if let Some(interval) = self.repeat_interval {
            false // Never finish if repeating
        } else {
            // Finish when pulse has moved beyond max distance
            let elapsed = self
                .start_time
                .get()
                .map_or(Duration::from_micros(0), |t| t);
            let distance_traveled = elapsed.as_micros() as f32 * 1e-6 * self.speed;
            distance_traveled > max_distance_from_center(&LED_POSITIONS_SORTED) * 2.0
        }
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

/// Animation that creates a static gradient based on distance from center
pub struct DistanceGradient {
    /// Color to use for the gradient
    pub color: Color,
    /// If true, LEDs fade out from center. If false, LEDs fade in from center
    pub fade_out: bool,
    /// Optional minimum brightness (0-150)
    pub min_brightness: u8,
    /// Optional maximum brightness (0-150)
    pub max_brightness: u8,
}

unsafe impl Sync for DistanceGradient {}

impl DistanceGradient {
    pub const fn new(color: Color, fade_out: bool, min_brightness: u8, max_brightness: u8) -> Self {
        Self {
            color,
            fade_out,
            min_brightness,
            max_brightness,
        }
    }
}

impl<const N: usize> Animation<N> for DistanceGradient {
    fn render(&self, _timestamp: Duration, buffer: &mut LedStateBuffer<N>) {
        let led_positions = &LED_POSITIONS_SORTED;
        let center = calculate_center(led_positions);
        let max_distance = max_distance_from_center(led_positions);

        // For each LED, calculate its brightness based on distance
        for (i, pos) in led_positions.iter().enumerate() {
            let distance = pos.distance_to(&center);

            // Normalize distance to 0-1 range
            let normalized_distance = distance / max_distance;

            // Calculate brightness based on fade direction
            let normalized_brightness = if self.fade_out {
                1.0 - normalized_distance
            } else {
                normalized_distance
            };

            // Scale brightness to min-max range and clamp
            let brightness_range = self.max_brightness.saturating_sub(self.min_brightness) as f32;
            let brightness = (normalized_brightness * brightness_range) as u8 + self.min_brightness;
            let brightness = brightness.min(150);

            // Set LED color with calculated brightness
            buffer.set_led(
                i,
                Color(
                    (self.color.0 as u32 * brightness as u32 / 150) as u8,
                    (self.color.1 as u32 * brightness as u32 / 150) as u8,
                    (self.color.2 as u32 * brightness as u32 / 150) as u8,
                ),
                Priority::Normal,
            );
        }
    }

    fn is_finished(&self) -> bool {
        false // Static pattern never finishes
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}

/// Animation that creates a warm, pulsing glow reminiscent of a sunset
pub struct SunsetGlow {
    time_offset: Cell<Duration>,
    base_color: Color,
    highlight_color: Color,
    pulse_speed: f32,
}

unsafe impl Sync for SunsetGlow {}

impl SunsetGlow {
    pub const fn new() -> Self {
        // Warm orange base with a redder highlight for the "sun" effect
        let base_color = Color(255, 100, 0); // Warm orange
        let highlight_color = Color(255, 30, 0); // Reddish

        Self {
            time_offset: Cell::new(Duration::from_millis(0)),
            base_color,
            highlight_color,
            pulse_speed: 3.0, // Faster to make movement more visible
        }
    }

    fn calculate_color(&self, distance_ratio: f32, pulse_intensity: f32) -> Color {
        // Make the blend more dramatic for visible changes
        let blend = (1.0 - distance_ratio * distance_ratio) * pulse_intensity;

        // Interpolate between colors with more contrast
        let base = if distance_ratio < 0.5 {
            self.highlight_color // Use highlight color in center
        } else {
            self.base_color // Use base color towards edges
        };

        let target = if distance_ratio < 0.5 {
            self.base_color // Pulse to base color in center
        } else {
            Color(255, 0, 0) // Pulse to deep red towards edges
        };

        Color(
            base.0
                .saturating_add(((target.0 as f32 - base.0 as f32) * blend) as u8),
            base.1.saturating_sub(((base.1 as f32) * blend) as u8),
            base.2
                .saturating_add(((target.2 as f32 - base.2 as f32) * blend) as u8),
        )
    }
}

impl<const N: usize> Animation<N> for SunsetGlow {
    fn reset(&self) {
        self.time_offset.set(Duration::from_millis(0));
    }

    fn render(&self, timestamp: Duration, buffer: &mut LedStateBuffer<N>) {
        let positions = &LED_POSITIONS_SORTED;

        // Update time offset
        self.time_offset.set(timestamp);
        let time = timestamp.as_millis() as f32 / 1000.0;

        // Create two overlapping waves with different frequencies
        let wave1 = sinf(time * self.pulse_speed) * 0.5 + 0.5;
        let wave2 = sinf(time * self.pulse_speed * 0.7 + 1.0) * 0.5 + 0.5;

        let center = calculate_center(positions);
        let max_distance = max_distance_from_center(positions);

        // Update all LEDs
        for (i, pos) in positions.iter().enumerate() {
            let distance = pos.distance_to(&center);
            let distance_ratio = distance / max_distance;

            // Combine waves with distance for more dynamic effect
            let pulse = wave1 * (1.0 - distance_ratio) + wave2 * distance_ratio;

            // Calculate color based on distance and pulse
            let color = self.calculate_color(distance_ratio, pulse);

            // Apply brightness with more contrast and minimum brightness
            let base_brightness = ((1.0 - distance_ratio) * 225.0) as u8;
            let brightness = base_brightness.saturating_add(30); // Ensure minimum brightness of 30

            // Apply brightness to each color component
            let dimmed_color = Color(
                ((color.0 as u32 * brightness as u32) / 255) as u8,
                ((color.1 as u32 * brightness as u32) / 255) as u8,
                ((color.2 as u32 * brightness as u32) / 255) as u8,
            );

            buffer.set_led(i, dimmed_color, Priority::Normal);
        }
    }

    fn is_finished(&self) -> bool {
        false // Continuous animation
    }

    fn priority(&self) -> Priority {
        Priority::Normal
    }
}
