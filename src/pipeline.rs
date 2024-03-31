use alsa::seq;

trait Filter {
    fn process(&self, event: Vec<seq::Event>) -> Vec<seq::Event>;
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
    _filters: Vec<Box<dyn Filter>>,
    /// Events taken in
    ingested: u32,
    /// Events written out
    delivered: u32,
}

impl Pipeline {
    pub fn new(sourceport: seq::Addr, sinkport: seq::Addr) -> Result<Pipeline, String> {
        use std::ffi::CString;

        // create sequencer for the pipeline
        let sequencer = match seq::Seq::open(None, None, false) {
            Ok(s) => s,
            Err(why) => return Err(format!("failed to create sequencer: {}", why)),
        };

        let _ = sequencer.set_client_name(&CString::new(format!("midirouter {}:{} to {}:{}", sourceport.client, sourceport.port, sinkport.client, sinkport.port)).unwrap());

        // create port to connect pipeline in and out
        let portname = CString::new(
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
            _filters: Vec::new(),
            ingested: 0,
            delivered: 0,
        })
    }

    pub fn run(&mut self) -> Result<bool, String> {
        use seq::EventType;

        let mut input = self.sequencer.input();
        match input.event_input_pending(true) {
            Ok(0) => return Ok(false),
            Err(why) => return Err(format!("failed processing pipeline: {}", why)),
            _ => (),
        }

        let mut event = match input.event_input() {
            Ok(event) => event.into_owned(),
            Err(why) => return Err(format!("failed processing pipeline: {}", why)),
        };
        event.set_direct();
        event.set_subs();
        self.ingested += 1;

        // FIXME: remove this and replace with a filter
        match event.get_type() {
            EventType::Sensing => return Ok(false),
            EventType::Clock => return Ok(false),
            _ => println!("{:?}", event),
        }
        // TODO: insert filter processing here

        match self.sequencer.event_output_direct(&mut event) {
            Ok(_) => {
                self.delivered += 1;
                Ok(true)
            },
            Err(why) => Err(format!("failed processing pipeline: {}", why)),
        }
    }

    pub fn get_status(&self) -> (u32, u32) {
        (self.ingested, self.delivered)
    }
}

impl Drop for Pipeline {
    fn drop(&mut self) {
        let _ = self.sequencer.unsubscribe_port(self.sourceport, self.myport);
        let _ = self.sequencer.unsubscribe_port(self.myport, self.sinkport);
    }
}
