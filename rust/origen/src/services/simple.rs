use crate::precludes::controller::*;
use crate::Result;

/// Simple dummy protocol
#[derive(Clone, Debug)]
pub struct Service {
    pub id: usize,
    pub clk: (String, usize),
    pub data: (String, usize),
    pub read_nwrite: (String, usize),
    pub width: usize,
}

impl ControllerAPI for Service {
    fn name(&self) -> String {
        "SimpleProtocol".to_string()
    }
}

impl Service {
    pub fn new(
        _dut: &Dut,
        id: usize,
        clk: &PinGroup,
        data: &PinGroup,
        read_nwrite: &PinGroup,
        width: usize,
    ) -> Result<Self> {
        Ok(Self {
            id: id,
            clk: clk.to_identifier(),
            data: data.to_identifier(),
            read_nwrite: read_nwrite.to_identifier(),
            width: width,
        })
    }

    fn _reset(
        &self,
        clk: &PinCollection,
        data: &PinCollection,
        read_nwrite: &PinCollection,
    ) -> Result<()> {
        clk.drive_low();
        data.highz();
        read_nwrite.highz().cycle();
        Ok(())
    }

    pub fn reset(&self, dut: &Dut) -> Result<()> {
        let clk = PinCollection::from_group(dut, &self.clk.0, self.clk.1)?;
        let data = PinCollection::from_group(dut, &self.data.0, self.data.1)?;
        let read_nwrite = PinCollection::from_group(dut, &self.read_nwrite.0, self.read_nwrite.1)?;

        let trans = node!(SimpleProtocolReset, self.id);
        let n_id = TEST.push_and_open(trans.clone());
        self.comment("Return SimpleProtocol to default pin states");
        self._reset(&clk, &data, &read_nwrite)?;
        TEST.close(n_id)?;
        Ok(())
    }

    pub fn write(&self, dut: &Dut, transaction: Transaction) -> Result<()> {
        let clk = PinCollection::from_group(dut, &self.clk.0, self.clk.1)?;
        let data = PinCollection::from_group(dut, &self.data.0, self.data.1)?;
        let read_nwrite = PinCollection::from_group(dut, &self.read_nwrite.0, self.read_nwrite.1)?;
        let mut t = transaction.clone();
        t.resize(self.width)?;
        t.apply_overlay_pin_ids(&data.as_ids())?;

        let trans = node!(SimpleProtocolWrite, self.id, t.clone());
        let n_id = TEST.push_and_open(trans.clone());
        self.comment(&format!("Simple Write: {} <- {}", t.addr()?, t.data));
        let addr_t = t.to_addr_trans(Some(self.width))?;
        self.comment(&format!(
            "Writing Address: {} (address width: {})",
            addr_t.data, addr_t.width
        ));
        clk.drive_high();
        read_nwrite.drive_low();
        data.push_transaction(&addr_t)?;
        self.comment(&format!(
            "Writing Data: {} (data width: {})",
            t.data, t.width
        ));
        data.push_transaction(&t)?;
        self._reset(&clk, &data, &read_nwrite)?;
        TEST.close(n_id)?;
        Ok(())
    }

    pub fn verify(&self, dut: &Dut, transaction: Transaction) -> Result<()> {
        let clk = PinCollection::from_group(dut, &self.clk.0, self.clk.1)?;
        let data = PinCollection::from_group(dut, &self.data.0, self.data.1)?;
        let read_nwrite = PinCollection::from_group(dut, &self.read_nwrite.0, self.read_nwrite.1)?;
        let mut t = transaction.clone();
        t.resize(self.width)?;
        t.apply_overlay_pin_ids(&data.as_ids())?;

        let trans = node!(SimpleProtocolWrite, self.id, t.clone());
        let n_id = TEST.push_and_open(trans.clone());
        self.comment(&format!("Simple Verify: {} =? {}", t.addr()?, t.data));
        let addr_t = t.to_addr_trans(Some(self.width))?;
        self.comment(&format!(
            "Verifying Address: {} (address width: {})",
            addr_t.data, addr_t.width
        ));
        clk.drive_high();
        read_nwrite.drive_high();
        data.push_transaction(&addr_t)?;
        self.comment(&format!(
            "Verifying Data: {} (data width: {})",
            t.data, t.width
        ));
        data.push_transaction(&t)?;
        self._reset(&clk, &data, &read_nwrite)?;
        TEST.close(n_id)?;
        Ok(())
    }
}
