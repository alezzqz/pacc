pub struct PaSource {
    pub sink_name: String,
    pub sink_description: String,
    pub port_name: String,
    pub port_description: String
}

impl PaSource {
    pub fn to_list_line(&self) -> String {
        format!("{}, Port '{}'",
            self.sink_description,
            self.port_description
        )
    }
}
