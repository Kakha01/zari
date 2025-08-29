use crate::engine::Scale;

pub struct Utils;

impl Utils {
    pub fn convert_sample_to_f64<T>(sample: T, scale: Scale) -> f64
    where
        T: Copy + Into<f64>,
    {
        (sample.into() * scale.get_f64_scale()).clamp(-1.0, 1.0)
    }

    pub fn convert_samples_to_f64<T>(samples: &[T], scale: Scale) -> Vec<f64>
    where
        T: Copy + Into<f64>,
    {
        samples
            .iter()
            .map(|sample| Self::convert_sample_to_f64(*sample, scale))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::i32;

    use super::*;

    #[test]
    fn test_convert_sample_i8_to_f64() {
        let scale = Scale::I8;

        assert_eq!(Utils::convert_sample_to_f64(0i8, scale), 0.0);
        assert_eq!(Utils::convert_sample_to_f64(i8::MAX, scale), 1.0);
        assert_eq!(Utils::convert_sample_to_f64(i8::MIN, scale), -1.0);

        let i8_max = i8::MAX as f64;

        assert!((Utils::convert_sample_to_f64(64i8, scale) - (64.0 / i8_max)).abs() < f64::EPSILON);
        assert!(
            (Utils::convert_sample_to_f64(-64i8, scale) - (-64.0 / i8_max)).abs() < f64::EPSILON
        );
        assert!((Utils::convert_sample_to_f64(1i8, scale) - (1.0 / i8_max)).abs() < f64::EPSILON);
        assert!((Utils::convert_sample_to_f64(-1i8, scale) - (-1.0 / i8_max)).abs() < f64::EPSILON);

        assert!((Utils::convert_sample_to_f64(32i8, scale) - (32.0 / i8_max)).abs() < f64::EPSILON);
        assert!(
            (Utils::convert_sample_to_f64(-32i8, scale) - (-32.0 / i8_max)).abs() < f64::EPSILON
        );

        assert!(
            (Utils::convert_sample_to_f64(126i8, scale) - (126.0 / i8_max)).abs() < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-127i8, scale) - (-127.0 / i8_max)).abs() < f64::EPSILON
        );
    }

    #[test]
    fn test_convert_sample_i16_to_f64() {
        let scale = Scale::I16;

        assert_eq!(Utils::convert_sample_to_f64(0i16, scale), 0.0);
        assert_eq!(Utils::convert_sample_to_f64(i16::MAX, scale), 1.0);
        assert_eq!(Utils::convert_sample_to_f64(i16::MIN, scale), -1.0);

        let i16_max = i16::MAX as f64;

        assert!(
            (Utils::convert_sample_to_f64(16384i16, scale) - (16384.0 / i16_max)).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-16384i16, scale) - (-16384.0 / i16_max)).abs()
                < f64::EPSILON
        );
        assert!((Utils::convert_sample_to_f64(1i16, scale) - (1.0 / i16_max)).abs() < f64::EPSILON);
        assert!(
            (Utils::convert_sample_to_f64(-1i16, scale) - (-1.0 / i16_max)).abs() < f64::EPSILON
        );

        assert!(
            (Utils::convert_sample_to_f64(8192i16, scale) - (8192.0 / i16_max)).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-8192i16, scale) - (-8192.0 / i16_max)).abs()
                < f64::EPSILON
        );

        assert!(
            (Utils::convert_sample_to_f64(32766i16, scale) - (32766.0 / i16_max)).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-32767i16, scale) - (-32767.0 / i16_max)).abs()
                < f64::EPSILON
        );

        assert!(
            (Utils::convert_sample_to_f64(1000i16, scale) - (1000.0 / i16_max)).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-1000i16, scale) - (-1000.0 / i16_max)).abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn test_convert_sample_i24_to_f64() {
        let scale = Scale::I24;

        assert_eq!(Utils::convert_sample_to_f64(0i32, scale), 0.0);
        assert_eq!(Utils::convert_sample_to_f64(8_388_607, scale), 1.0);
        assert_eq!(Utils::convert_sample_to_f64(-8_388_608, scale), -1.0);

        let i24_max = 8_388_607.0;

        assert!(
            (Utils::convert_sample_to_f64(4_194_304i32, scale) - (4_194_304.0 / i24_max)).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-4_194_304i32, scale) - (-4_194_304.0 / i24_max)).abs()
                < f64::EPSILON
        );
        assert!((Utils::convert_sample_to_f64(1i32, scale) - (1.0 / i24_max)).abs() < f64::EPSILON);
        assert!(
            (Utils::convert_sample_to_f64(-1i32, scale) - (-1.0 / i24_max)).abs() < f64::EPSILON
        );

        assert!(
            (Utils::convert_sample_to_f64(2_097_152i32, scale) - (2_097_152.0 / i24_max)).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-2_097_152i32, scale) - (-2_097_152.0 / i24_max)).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(1_048_576i32, scale) - (1_048_576.0 / i24_max)).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-1_048_576i32, scale) - (-1_048_576.0 / i24_max)).abs()
                < f64::EPSILON
        );

        assert!(
            (Utils::convert_sample_to_f64(8_388_606i32, scale) - (8_388_606.0 / i24_max)).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-8_388_607i32, scale) - (-8_388_607.0 / i24_max)).abs()
                < f64::EPSILON
        );

        assert!(
            (Utils::convert_sample_to_f64(100_000i32, scale) - (100_000.0 / i24_max)).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-100_000i32, scale) - (-100_000.0 / i24_max)).abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn test_convert_sample_i32_to_f64() {
        let scale = Scale::I32;

        assert_eq!(Utils::convert_sample_to_f64(0i32, scale), 0.0);
        assert_eq!(Utils::convert_sample_to_f64(i32::MAX, scale), 1.0);
        assert_eq!(Utils::convert_sample_to_f64(i32::MIN, scale), -1.0);

        let i32_max = i32::MAX as f64;

        assert!(
            (Utils::convert_sample_to_f64(1_073_741_824i32, scale) - (1_073_741_824.0 / i32_max))
                .abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-1_073_741_824i32, scale) - (-1_073_741_824.0 / i32_max))
                .abs()
                < f64::EPSILON
        );
        assert!((Utils::convert_sample_to_f64(1i32, scale) - (1.0 / i32_max)).abs() < f64::EPSILON);
        assert!(
            (Utils::convert_sample_to_f64(-1i32, scale) - (-1.0 / i32_max)).abs() < f64::EPSILON
        );

        assert!(
            (Utils::convert_sample_to_f64(536_870_912i32, scale) - (536_870_912.0 / i32_max)).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-536_870_912i32, scale) - (-536_870_912.0 / i32_max))
                .abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(268_435_456i32, scale) - (268_435_456.0 / i32_max)).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-268_435_456i32, scale) - (-268_435_456.0 / i32_max))
                .abs()
                < f64::EPSILON
        );

        assert!(
            (Utils::convert_sample_to_f64(2_147_483_646i32, scale) - (2_147_483_646.0 / i32_max))
                .abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-2_147_483_647i32, scale) - (-2_147_483_647.0 / i32_max))
                .abs()
                < f64::EPSILON
        );

        assert!(
            (Utils::convert_sample_to_f64(10_000_000i32, scale) - (10_000_000.0 / i32_max)).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-10_000_000i32, scale) - (-10_000_000.0 / i32_max)).abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn test_convert_sample_f32_to_f64() {
        let scale = Scale::F32;

        assert_eq!(Utils::convert_sample_to_f64(0.0f32, scale), 0.0);
        assert_eq!(Utils::convert_sample_to_f64(1.0f32, scale), 1.0);
        assert_eq!(Utils::convert_sample_to_f64(-1.0f32, scale), -1.0);

        assert!((Utils::convert_sample_to_f64(0.5f32, scale) - 0.5).abs() < f64::EPSILON);
        assert!((Utils::convert_sample_to_f64(-0.5f32, scale) - (-0.5)).abs() < f64::EPSILON);
        assert!((Utils::convert_sample_to_f64(0.25f32, scale) - 0.25).abs() < f64::EPSILON);
        assert!((Utils::convert_sample_to_f64(-0.25f32, scale) - (-0.25)).abs() < f64::EPSILON);

        assert!(
            (Utils::convert_sample_to_f64(0.9999f32, scale) - 0.9999f32 as f64).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-0.9999f32, scale) - (-0.9999f32 as f64)).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(0.0001f32, scale) - 0.0001f32 as f64).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-0.0001f32, scale) - (-0.0001f32 as f64)).abs()
                < f64::EPSILON
        );

        assert!(
            (Utils::convert_sample_to_f64(0.123456f32, scale) - 0.123456f32 as f64).abs()
                < f64::EPSILON
        );
        assert!(
            (Utils::convert_sample_to_f64(-0.789012f32, scale) - (-0.789012f32 as f64)).abs()
                < f64::EPSILON
        );
    }

    #[test]
    fn test_convert_samples_i8_to_f64() {
        let scale = Scale::I8;
        let samples = vec![0i8, i8::MAX, i8::MIN, 64, -64];
        let expected = vec![
            0.0,
            1.0,
            -1.0,
            64.0 / i8::MAX as f64,
            -64.0 / i8::MAX as f64,
        ];

        let result = Utils::convert_samples_to_f64(&samples, scale);

        for (actual_val, expected_val) in result.iter().zip(expected) {
            assert!((actual_val - expected_val).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_convert_samples_i16_to_f64() {
        let scale = Scale::I16;
        let samples = vec![0i16, i16::MAX, i16::MIN, 16384, -16384];
        let expected = vec![
            0.0,
            1.0,
            -1.0,
            16384.0 / i16::MAX as f64,
            -16384.0 / i16::MAX as f64,
        ];

        let result = Utils::convert_samples_to_f64(&samples, scale);

        for (actual_val, expected_val) in result.iter().zip(expected) {
            assert!((actual_val - expected_val).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_convert_samples_i24_to_f64() {
        let scale = Scale::I24;
        let samples = vec![0i32, 8_388_607, -8_388_608, 4_194_304, -4_194_304];
        let expected = vec![
            0.0,
            1.0,
            -1.0,
            4_194_304.0 / 8_388_607.0,
            -4_194_304.0 / 8_388_607.0,
        ];

        let result = Utils::convert_samples_to_f64(&samples, scale);

        for (actual_val, expected_val) in result.iter().zip(expected) {
            assert!((actual_val - expected_val).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_convert_samples_i32_to_f64() {
        let scale = Scale::I32;
        let samples = vec![0i32, i32::MAX, i32::MIN, 1_073_741_824, -1_073_741_824];
        let expected = vec![
            0.0,
            1.0,
            -1.0,
            1_073_741_824.0 / i32::MAX as f64,
            -1_073_741_824.0 / i32::MAX as f64,
        ];

        let result = Utils::convert_samples_to_f64(&samples, scale);

        for (actual_val, expected_val) in result.iter().zip(expected) {
            assert!((actual_val - expected_val).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_convert_samples_f32_to_f64() {
        let scale = Scale::F32;
        let samples = vec![0.0f32, 1.0, -1.0, 0.5, -0.5];
        let expected = vec![0.0, 1.0, -1.0, 0.5, -0.5];

        let result = Utils::convert_samples_to_f64(&samples, scale);

        for (actual_val, expected_val) in result.iter().zip(expected) {
            assert!((actual_val - expected_val).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_convert_samples_empty_slice() {
        let scale = Scale::I16;
        let samples: Vec<i16> = vec![];
        let expected: Vec<f64> = vec![];

        let result = Utils::convert_samples_to_f64(&samples, scale);

        assert_eq!(result, expected);
    }

    #[test]
    fn test_convert_samples_f32_clamping() {
        let scale = Scale::F32;
        let samples = vec![2.0f32, -2.0, 1.5, -1.5, 0.0];
        let expected = vec![1.0, -1.0, 1.0, -1.0, 0.0];

        let result = Utils::convert_samples_to_f64(&samples, scale);

        for (actual_val, expected_val) in result.iter().zip(expected) {
            assert!((actual_val - expected_val).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_convert_samples_single_element() {
        let scale = Scale::I8;
        let samples = vec![i8::MAX];
        let expected = vec![1.0];

        let result = Utils::convert_samples_to_f64(&samples, scale);

        for (actual_val, expected_val) in result.iter().zip(expected) {
            assert!((actual_val - expected_val).abs() < f64::EPSILON);
        }
    }

    #[test]
    fn test_convert_samples_large_slice() {
        let scale = Scale::I16;
        let samples: Vec<i16> = vec![1000; 10000];
        let expected_value = 1000.0 / i16::MAX as f64;

        let result = Utils::convert_samples_to_f64(&samples, scale);

        assert_eq!(result.len(), 10000);
        for actual_val in result.iter() {
            assert!((actual_val - expected_value).abs() < f64::EPSILON);
        }
    }
}
