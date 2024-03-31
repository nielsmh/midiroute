use alsa::seq;

trait Filter {
    fn process(&self, event: &seq::Event) -> Vec<seq::Event>;
}

pub struct Pipeline {
    /// The sequencer object running this pipeline
    sequencer: seq::Seq,
    /// Remote port address the input stream originates from
    sourceport: seq::Addr,
    /// Remote port address the output stream is delivered to
    sinkport: seq::Addr,
    /// Own port address used for reading and writing events
    myport: seq::Addr,
    /// Sequence of filters to apply to input
    filters: Vec<Box<dyn Filter>>,
}

impl Pipeline {
    pub fn new(sourceport: seq::Addr, sinkport: seq::Addr) -> Result<Pipeline, String> {
        // create sequencer for the pipeline
        let sequencer = match seq::Seq::open(None, None, false) {
            Ok(s) => s,
            Err(why) => return Err(format!("failed to create sequencer: {}", why)),
        };

        // create port to connect pipeline in and out
        let portname = std::ffi::CString::new(
            format!("pipeline {}:{} to {}:{}", sourceport.client, sourceport.port, sinkport.client, sinkport.port)
        ).unwrap();

        let mut myportinfo = seq::PortInfo::empty().unwrap();
        myportinfo.set_capability(seq::PortCap::WRITE | seq::PortCap::SUBS_WRITE);
        myportinfo.set_type(seq::PortType::MIDI_GENERIC | seq::PortType::APPLICATION);
        myportinfo.set_name(&portname);
        let myport = match sequencer.create_port(&myportinfo) {
            Ok(_) => myportinfo.addr(),
            Err(why) => return Err(format!("failed to create port: {}", why)),
        };

        // connect sourceport to myport
        let sourcesub = seq::PortSubscribe::empty().unwrap();
        sourcesub.set_sender(sourceport);
        sourcesub.set_dest(myport);
        sourcesub.set_time_real(true);
        if let Err(why) = sequencer.subscribe_port(&sourcesub) {
            return Err(format!("failed to subscribe self to source: {}", why));
        }

        // connect myport to sinkport
        let sinksub = seq::PortSubscribe::empty().unwrap();
        sinksub.set_sender(myport);
        sinksub.set_dest(sinkport);
        sourcesub.set_time_real(true);
        if let Err(why) = sequencer.subscribe_port(&sinksub) {
            return Err(format!("failed to subscribe sink to self: {}", why));
        }

        // finished
        Ok(Pipeline {
            sequencer: sequencer,
            sourceport: sourceport,
            sinkport: sinkport,
            myport: myport,
            filters: Vec::new(),
        })
    }
}
