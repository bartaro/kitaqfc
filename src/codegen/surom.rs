use super::*;

impl Generator {
    pub(super) fn validate_surom_interrupts(&mut self) {
        if !self.profile.is_surom() {
            return;
        }
        let mut reachable = BTreeSet::new();
        let mut pending = Vec::new();
        for root in ["__nes_nmi", "__nes_irq"] {
            if self.functions.contains_key(root) {
                reachable.insert(root.to_owned());
                pending.push(root.to_owned());
            }
        }
        while let Some(caller) = pending.pop() {
            for (from, to) in &self.call_edges {
                if *from == caller && reachable.insert(to.clone()) {
                    pending.push(to.clone());
                }
            }
        }
        for name in reachable {
            if self.mapper_writers.contains(&name) {
                self.errors.push(format!("error KQFC2606 KQFC-SUROM-NMI-MAPPER-WRITE: MMC1 register write is reachable from interrupt function '{name}' on board 'surom512'"));
            }
            if self.functions.get(&name).map_or(1, |f| f.bank) != 0 {
                self.errors.push(format!("error KQFC2607: interrupt-reachable function '{name}' must remain in common bank 0 on board 'surom512'"));
            }
        }
    }
}
