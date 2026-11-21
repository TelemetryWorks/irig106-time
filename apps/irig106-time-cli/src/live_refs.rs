//! Channel-local ages under a shared 1024-arrival resource budget.
//! Empty caches retain loss history; channel IDs bound metadata, not file size.
use irig106_time::NanosDuration;
use irig106_time::{AbsoluteTime, Rtc, StreamingTimeCorrelator};
use std::collections::{BTreeMap, VecDeque};

#[derive(Default)]
pub(super) struct LiveReferences {
    channels: BTreeMap<u16, StreamingTimeCorrelator>,
    order: VecDeque<(u16, Rtc)>,
}
impl LiveReferences {
    pub(super) fn observe(&mut self, ch: u16, rtc: Rtc, time: AbsoluteTime) {
        self.channels
            .entry(ch)
            .or_insert_with(|| StreamingTimeCorrelator::new(NanosDuration::new(30_000_000_000)))
            .add_reference(ch, rtc, time);
        self.order.push_back((ch, rtc));
        if self.order.len() > super::REFERENCE_CAP.get() {
            if let Some((channel, old)) = self.order.pop_front() {
                if let Some(cache) = self.channels.get_mut(&channel) {
                    cache.discard_reference(channel, old);
                }
            }
        }
    }
    #[cfg(test)]
    pub(super) fn counts(&self) -> (usize, usize) {
        (
            self.channels
                .values()
                .map(StreamingTimeCorrelator::len)
                .sum(),
            self.order.len(),
        )
    }
    pub(super) fn clear(&mut self, ch: u16) {
        if let Some(cache) = self.channels.get_mut(&ch) {
            cache.clear_channel(ch);
        }
        self.order.retain(|(channel, _)| *channel != ch);
    }
    pub(super) fn correlate(&self, rtc: Rtc, ch: u16) -> irig106_time::Result<AbsoluteTime> {
        self.channels
            .get(&ch)
            .ok_or(irig106_time::TimeError::NoReferencePoint)?
            .correlate(rtc, Some(ch))
    }
}
