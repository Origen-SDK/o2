use crate::Result;

pub trait NumHelpers {
    type T;

    fn even_parity(&self) -> bool;
    // fn reverse(&self, width: usize) -> Result<Self::T>;
    // fn chunk(
    //     &self,
    //     chunk_width: usize,
    //     data_width: usize,
    //     chunk_lsb_first: bool,
    //     data_lsb_first: bool
    // ) -> crate::Result<Vec<Self::T>>;
}

impl NumHelpers for u32 {
    type T = Self;

    fn even_parity(&self) -> bool {
        if self.count_ones() % 2 == 0 {
            false
        } else {
            true
        }
    }

    // fn reverse(&self, width: usize) -> Result<Self::T> {
    //     Ok(self.reverse_bits())
    // }

    // fn chunk(
    //     &self,
    //     chunk_width: usize,
    //     data_width: usize,
    //     chunk_lsb_first: bool,
    //     data_lsb_first: bool
    // ) -> Result<Vec<Self::T>> {
    //     let mut data = self.clone();
    //     let retn = vec!();
    //     let mask = (1 << chunk_width - 1) as Self;
    //     for i in 0..(data_width/chunk_width) {
    //         retn.push(data & mask);
    //         data >> chunk_width;
    //     }
    //     if data_width % chunk_width != 0 {
    //         retn.push(data);
    //     }
    //     Ok(retn)
    // }
}
