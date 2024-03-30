use alsa::{
    rawmidi,
    card,
};

fn main() {
    use alsa::Ctl;

    for a in card::Iter::new().map(|a| a.unwrap()) {
        for b in rawmidi::Iter::new(&Ctl::from_card(&a, false).unwrap()).map(|b| b.unwrap()) {
            println!("Rawmidi {:?} (hw:{},{},{}) {} - {}", b.get_stream(), a.get_index(), b.get_device(), b.get_subdevice(),
                 a.get_name().unwrap(), b.get_subdevice_name().unwrap())
        }
    }
}
