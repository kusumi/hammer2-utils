use crate::env;
use crate::show;
use crate::Hammer2Options;
use hammer2_utils::hammer2fs;
use hammer2_utils::ondisk;
use hammer2_utils::util;
use hammer2_utils::volume;

pub(crate) fn run(devpath: &str, opt: &Hammer2Options) -> std::io::Result<()> {
    let t = env::init();
    let sopt = show::ShowOptions::new(t.0, t.1, t.2, t.3, t.4, 0);

    let mut fso = ondisk::init(devpath, true)?;
    let best = fso.get_best_volume_data()?[hammer2fs::HAMMER2_ROOT_VOLUME as usize];

    println!(
        "{}",
        fso.get_root_volume().ok_or_else(util::notfound)?.get_path()
    );
    for i in 0..hammer2fs::HAMMER2_NUM_VOLHDRS {
        let vol = fso.get_root_volume_mut().ok_or_else(util::notfound)?;
        let offset = volume::get_volume_data_offset(i);
        if offset < vol.get_size() {
            let buf = vol.preadx(hammer2fs::HAMMER2_VOLUME_BYTES, offset)?;
            let voldata = util::align_to::<hammer2fs::Hammer2VolumeData>(&buf);
            show::print_volume_summary(
                hammer2fs::HAMMER2_ROOT_VOLUME.into(),
                i,
                voldata.mirror_tid,
            );
            if sopt.all_volume_data || best.0 == i {
                let mut broot =
                    hammer2fs::Hammer2Blockref::new(hammer2fs::HAMMER2_BREF_TYPE_VOLUME);
                broot.mirror_tid = voldata.mirror_tid;
                broot.data_off = offset | u64::try_from(hammer2fs::HAMMER2_PBUFRADIX).unwrap();
                show::show_blockref(
                    &mut fso,
                    voldata,
                    sopt.init_tab,
                    i,
                    &broot,
                    false,
                    &mut None,
                    &sopt,
                    opt,
                )?;
            }
            if sopt.all_volume_data && i != hammer2fs::HAMMER2_NUM_VOLHDRS - 1 {
                println!();
            }
        }
    }
    Ok(())
}
