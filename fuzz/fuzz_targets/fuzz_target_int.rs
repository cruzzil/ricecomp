#![no_main]
use libfuzzer_sys::fuzz_target;
extern crate ricecomp;
use ricecomp::read::RCDecoder;
use ricecomp::write::RCEncoder;

#[derive(Clone, Debug, arbitrary::Arbitrary)]
pub struct DataInt {
    pub d: Vec<i32>,
}

fuzz_target!(|data: DataInt| {
    let l = data.d.len();
   // let blocksz = if data.bs > 64 { (data.bs % 64)+1 } else {data.bs};
    //let blocksz = if blocksz < 1 { 1 } else { blocksz };
    let blocksz = 32;


    let mut comp_array = Vec::new();
    let mut encoder = RCEncoder::new(&mut comp_array);
    let out_count = encoder.encode(&data.d, l, blocksz as usize);

    match out_count {
        Ok(_) => {
            let decoder = RCDecoder::new();
            let mut decomp_array = vec![0; l];
            decoder.decode(&comp_array, l, blocksz as usize, &mut decomp_array).unwrap();
            let decomp_array: Vec<i32> = decomp_array.iter().map(|&x| x as i32).collect();

            assert_eq!(data.d, decomp_array);
        },

        Err(_) => {

        }
    }
    
});
