pub struct SystemConfig {}

impl SystemConfig {
    pub fn new() -> Self {
        // TODO read+report folder where game files will be searched from
        if let Some(proj_dir_obj) =
            directories::ProjectDirs::from("com.jmcateer", "FullCrisis", "FullCrisis")
        {
            eprintln!(
                "proj_dir_obj.config_local_dir() = {:?}",
                proj_dir_obj.config_local_dir()
            );
            eprintln!("proj_dir_obj.data_dir() = {:?}", proj_dir_obj.data_dir());
        }

        if let Some(locale_bcp_47) = sys_locale::get_locale() {
            eprintln!("locale_bcp_47 = {:?}", locale_bcp_47);
            // Go from the first 2 chars, which are ISO-639 2-letter language codes, and get the ISO-639 3-letter code.0
            if let Some(lang_639) = rust_iso639::from_code_1(&locale_bcp_47[..2]) {
                eprintln!("lang_639.code_3 = {:?}", lang_639.code_3);
            }
        }

        Self {}
    }
}
