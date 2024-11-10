/*++ @file

    Copyright ©2024-2024 Liu Yi, efikarl@yeah.net

    This program is just made available under the terms and conditions of the
    MIT license: http://www.efikarl.com/mit-license.html

    THE PROGRAM IS DISTRIBUTED UNDER THE MIT LICENSE ON AN "AS IS" BASIS,
    WITHOUT WARRANTIES OR REPRESENTATIONS OF ANY KIND, EITHER EXPRESS OR IMPLIED.
--*/

pub trait PathEx {
    fn to_string(&self) -> String;
    fn try_create_parent(&self, file: bool) -> Result<std::path::PathBuf, std::io::Error>;
}

impl<P: AsRef<std::path::Path>> PathEx for P {
    fn to_string(&self) -> String {
        self.as_ref().to_string_lossy().into_owned()
    }

    fn try_create_parent(&self, file: bool) -> Result<std::path::PathBuf, std::io::Error> {
        let     path    = std::path::absolute(self)?;
        let mut parent  = std::path::PathBuf::new();

        let components  = path.components().collect::<Vec<_>>();
        let length      = components.len();
        for (i , item) in components.into_iter().enumerate() {
            if file && (i == length - 1) {
                break;
            }
            parent = parent.join(item.as_os_str());
            if parent.is_dir() {
                continue;
            } else {
                std::fs::create_dir(&parent)?;
            }
        }

        Ok(std::path::PathBuf::from(path))
    }
}
