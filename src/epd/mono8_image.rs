use crate::*;
use eight_px_uint_eight::unix::EightDataClient;
use eight_px_uint_eight::*;
use itertools::Itertools;
use std::cmp::max;

pub struct EpdMonoColorImage {
    pub rect: EightSizedRectangle,
    pub data: HorizontalEightPxUintEight<EightDataClient>,
}

impl EpdMonoColorImage {
    pub fn new(rect: NormalRectangle, data: &[impl ActAsMono]) -> EpdResult<Self> {
        unimplemented!()
    }

    pub fn new_with_eight(rect: EightSizedRectangle, data: Vec<u8>) -> EpdResult<Self> {
        unimplemented!()
    }
}

impl ActAsMono for EpdColor {
    fn act_as(&self) -> Mono {
        match self {
            EpdColor::Black => Mono::Zero,
            EpdColor::White => Mono::One,
        }
    }
}

impl ActAsXywh for NormalRectangle {
    fn xywh(&self) -> (usize, usize, usize, usize) {
        let NormalRectangle {
            x,
            y,
            width,
            height,
            ..
        } = *self;
        (x as usize, y as usize, width as usize, height as usize)
    }
}

impl From<eight_px_uint_eight::Rectangle> for EightSizedRectangle {
    fn from(rect: eight_px_uint_eight::Rectangle) -> Self {
        let eight_px_uint_eight::Rectangle {
            x,
            y,
            width,
            height,
        } = rect;
        Self::new(x as u16, y as u16, width as u16, height as u16)
    }
}

impl EpdImage for EpdMonoColorImage {
    fn rect(&self) -> &EightSizedRectangle {
        &self.rect
    }

    fn update(&mut self, rect: NormalRectangle, colors: &[EpdColor]) -> EpdResult<()> {
        unimplemented!()
    }

    fn as_vec(&self) -> &[u8] {
        self.data.as_vec()
    }

    fn as_part_vec(&self, rect: NormalRectangle) -> (EightSizedRectangle, Vec<u8>) {
        unimplemented!()
    }

    fn data_for_fill(rect: NormalRectangle, color: EpdColor) -> EpdResult<Self> {
        unimplemented!()
    }
}
