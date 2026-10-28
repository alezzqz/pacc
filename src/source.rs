use libpulse_binding::volume::{ChannelVolumes, Volume};

#[derive(Debug)]
pub struct PaOutput {
    pub is_active_port: bool,
    pub sink_name: String,
    pub sink_description: String,
    pub port_name: String,
    pub port_description: String,
    pub volume: ChannelVolumes,
    pub volume_known: bool,
    pub pending_volume: bool,
}

impl PaOutput {
    pub fn volume_percent(&self) -> u32 {
        ((self.volume.max().0 as u64 * 100 + Volume::NORMAL.0 as u64 / 2)
            / Volume::NORMAL.0 as u64) as u32
    }

    pub fn set_volume_percent(&mut self, percent: u32) -> Result<(), &'static str> {
        let level = Volume((Volume::NORMAL.0 as u64 * percent as u64 / 100) as u32);
        self.volume.scale(level).ok_or("can't scale sink volume")?;
        Ok(())
    }

    pub fn to_list_line(&self) -> String {
        format!("{}, Port '{}'",
            self.sink_description,
            self.port_description
        )
    }
}
