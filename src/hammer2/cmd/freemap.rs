pub(crate) fn run(devpath: &str, opt: &crate::Opt) -> hammer2_utils::Result<()> {
    let t = crate::env::init();
    let sopt = crate::show::ShowOptions::new(t.0, t.1, t.2, t.3, t.4, 0);

    let mut fso = libhammer2::ondisk::init(devpath, true)?;
    let best = fso.get_best_volume_data()?[libhammer2::fs::HAMMER2_ROOT_VOLUME as usize];
    let mut stat = Some(crate::show::FreemapStat::new());

    println!(
        "{}",
        fso.get_root_volume()
            .ok_or(nix::errno::Errno::ENODEV)?
            .get_path()
    );
    for i in 0..libhammer2::fs::HAMMER2_NUM_VOLHDRS {
        let vol = fso.get_root_volume_mut().ok_or(nix::errno::Errno::ENODEV)?;
        let offset = libhammer2::volume::get_volume_data_offset(i);
        if offset < vol.get_size() {
            let buf = vol.preadx(libhammer2::fs::HAMMER2_VOLUME_BYTES, offset)?;
            let voldata = libhammer2::ondisk::media_as_volume_data(&buf);
            crate::show::print_volume_summary(
                libhammer2::fs::HAMMER2_ROOT_VOLUME.into(),
                i,
                voldata.mirror_tid,
            );
            if sopt.all_volume_data || best.0 == i {
                let mut broot =
                    libhammer2::fs::Hammer2Blockref::new(libhammer2::fs::HAMMER2_BREF_TYPE_FREEMAP);
                broot.mirror_tid = voldata.mirror_tid;
                broot.data_off = offset | u64::try_from(libhammer2::fs::HAMMER2_PBUFRADIX)?;
                crate::show::show_blockref(
                    &mut fso,
                    voldata,
                    sopt.init_tab,
                    i,
                    &broot,
                    false,
                    &mut stat,
                    &sopt,
                    opt,
                )?;
            }
            if sopt.all_volume_data && i != libhammer2::fs::HAMMER2_NUM_VOLHDRS - 1 {
                println!();
            }
        }
    }

    if let Some(stat) = stat {
        println!(
            "Total unallocated storage:   {:6.3}GB ({:6.3}GB in 64KB chunks)",
            stat.accum16[0] as f64 / libhammer2::subs::G_F64,
            stat.accum64[0] as f64 / libhammer2::subs::G_F64
        );
        println!(
            "Total possibly free storage: {:6.3}GB ({:6.3}GB in 64KB chunks)",
            stat.accum16[2] as f64 / libhammer2::subs::G_F64,
            stat.accum64[2] as f64 / libhammer2::subs::G_F64
        );
        println!(
            "Total allocated storage:     {:6.3}GB ({:6.3}GB in 64KB chunks)",
            stat.accum16[3] as f64 / libhammer2::subs::G_F64,
            stat.accum64[3] as f64 / libhammer2::subs::G_F64
        );
        println!(
            "Total unavailable storage:   {:6.3}GB",
            stat.unavail as f64 / libhammer2::subs::G_F64
        );
        println!(
            "Total freemap storage:       {:6.3}GB",
            stat.freemap as f64 / libhammer2::subs::G_F64
        );
    }
    Ok(())
}
