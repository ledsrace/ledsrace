use ledsrace_logic::{Circuit, Color, LedStateBuffer, Point, Priority, Sector};

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

pub struct Zandvoort<const N: usize> {
    buffer: LedStateBuffer<N>,
}

impl<const N: usize> Zandvoort<N> {
    pub fn new() -> Self {
        Self {
            buffer: LedStateBuffer::new(),
        }
    }
}

impl<const N: usize> Circuit<N> for Zandvoort<N> {
    const LED_COUNT: usize = 216;

    fn led_positions(&self) -> &'static [Point] {
        &LED_POSITIONS_SORTED
    }

    fn led_count(&self) -> usize {
        Self::LED_COUNT
    }

    fn sectors(&self, sector: Sector) -> &'static [Point] {
        &self.led_positions()[self.sector_indices(sector)]
    }

    fn sector_indices(&self, sector: Sector) -> core::ops::Range<usize> {
        match sector {
            Sector::_1 => 0..77,
            Sector::_2 => 77..153,
            Sector::_3 => 153..216,
        }
    }

    fn led_buffer(&mut self) -> &mut LedStateBuffer<N> {
        &mut self.buffer
    }

    fn set_led(&mut self, index: usize, color: Color, priority: Priority) {
        self.buffer.set_led(index, color, priority);
    }
}
