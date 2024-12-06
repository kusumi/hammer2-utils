use crate::env;
use crate::show;
use crate::Hammer2Options;

pub(crate) fn run(devpath: &str, opt: &Hammer2Options) -> Result<(), Box<dyn std::error::Error>> {
    let t = env::init();
    let sopt = show::ShowOptions::new(t.0, t.1, t.2, t.3, t.4, 16);

    let mut fso = libhammer2::ondisk::init(devpath, true)?;
    let bests = fso.get_best_volume_data()?;
    let n = fso.get_nvolumes().into();

    for i in 0..n {
        println!("{}", fso[i].get_path());
        for j in 0..libhammer2::fs::HAMMER2_NUM_VOLHDRS {
            let vol = &mut fso[i];
            let offset = libhammer2::volume::get_volume_data_offset(j);
            if offset < vol.get_size() {
                let buf = vol.preadx(libhammer2::fs::HAMMER2_VOLUME_BYTES, offset)?;
                let voldata = libhammer2::util::align_to::<libhammer2::fs::Hammer2VolumeData>(&buf);
                show::print_volume_summary(i, j, voldata.mirror_tid);
                if sopt.all_volume_data || bests[i].0 == j {
                    show::show_volume_data(&mut fso, voldata, j, &sopt, opt)?;
                }
                if sopt.all_volume_data && j != libhammer2::fs::HAMMER2_NUM_VOLHDRS - 1 {
                    println!();
                }
            }
        }
        if i != n - 1 {
            println!("---------------------------------------------");
        }
    }
    Ok(())
}
