use bitcoin::{Amount, ScriptBuf, TxOut};
use miniscript::bitcoin;

use crate::DefiniteDescriptor;

/// Can get script pubkey from this.
#[derive(Debug, Clone)]
pub enum ScriptSource {
    /// From ScriptBuf.
    Script(ScriptBuf),
    /// From definite descriptor.
    Descriptor(DefiniteDescriptor),
}

impl From<ScriptBuf> for ScriptSource {
    fn from(script: ScriptBuf) -> Self {
        Self::from_script(script)
    }
}

impl From<DefiniteDescriptor> for ScriptSource {
    fn from(descriptor: DefiniteDescriptor) -> Self {
        Self::from_descriptor(descriptor)
    }
}

impl ScriptSource {
    /// From script
    pub fn from_script(script: ScriptBuf) -> Self {
        Self::Script(script)
    }

    /// From descriptor
    pub fn from_descriptor(descriptor: DefiniteDescriptor) -> Self {
        Self::Descriptor(descriptor)
    }

    /// To ScriptBuf
    pub fn script(&self) -> ScriptBuf {
        match self {
            ScriptSource::Script(script_buf) => script_buf.clone(),
            ScriptSource::Descriptor(descriptor) => descriptor.script_pubkey(),
        }
    }

    /// Get descriptor (if any).
    pub fn descriptor(&self) -> Option<&DefiniteDescriptor> {
        match self {
            ScriptSource::Script(_) => None,
            ScriptSource::Descriptor(descriptor) => Some(descriptor),
        }
    }
}

/// Builder output
#[derive(Debug, Clone)]
pub struct Output {
    /// Value
    pub value: Amount,
    /// Spk source
    pub script_pubkey_source: ScriptSource,
}

impl From<(ScriptBuf, Amount)> for Output {
    fn from((script, value): (ScriptBuf, Amount)) -> Self {
        Self::with_script(script, value)
    }
}

impl From<(DefiniteDescriptor, Amount)> for Output {
    fn from((descriptor, value): (DefiniteDescriptor, Amount)) -> Self {
        Self::with_descriptor(descriptor, value)
    }
}

impl Output {
    /// From script
    pub fn with_script(script: ScriptBuf, value: Amount) -> Self {
        Self {
            value,
            script_pubkey_source: script.into(),
        }
    }

    /// From descriptor
    pub fn with_descriptor(descriptor: DefiniteDescriptor, value: Amount) -> Self {
        Self {
            value,
            script_pubkey_source: descriptor.into(),
        }
    }

    /// Script pubkey
    pub fn script_pubkey(&self) -> ScriptBuf {
        self.script_pubkey_source.script()
    }

    /// Descriptor
    pub fn descriptor(&self) -> Option<&DefiniteDescriptor> {
        self.script_pubkey_source.descriptor()
    }

    /// Create txout.
    pub fn txout(&self) -> TxOut {
        TxOut {
            value: self.value,
            script_pubkey: self.script_pubkey_source.script(),
        }
    }

    /// To coin select drain (change) output weights.
    ///
    /// Returns `None` if no descriptor is avaliable or the output is unspendable.
    pub fn to_drain_weights(&self) -> Option<bdk_coin_select::DrainWeights> {
        let descriptor = self.descriptor()?;
        Some(bdk_coin_select::DrainWeights {
            output_weight: self.txout().weight().to_wu(),
            spend_weight: descriptor.max_weight_to_satisfy().ok()?.to_wu(),
            n_outputs: 1,
        })
    }

    /// To coin select target outputs.
    pub fn to_target_outputs(&self) -> bdk_coin_select::TargetOutputs {
        bdk_coin_select::TargetOutputs {
            value_sum: self.txout().value.to_sat(),
            weight_sum: self.txout().weight().to_wu(),
            n_outputs: 1,
        }
    }
}
