//// Aggregate Source

use core::{MetricSink, MetricType, MetricWriter, MetricSource};
use aggregate::sink::{ScoreIterator, AggregateScore};

/// publisher from aggregate metrics to target channel
pub struct AggregateSource<C: MetricSink> {
    target: C,
    scores: ScoreIterator,
}

impl <C: MetricSink> AggregateSource<C> {

    /// create new publisher from aggregate metrics to target channel
    pub fn new(target: C, scores: ScoreIterator) -> AggregateSource<C> {
        AggregateSource {target, scores}
    }
}

impl <C: MetricSink> MetricSource for AggregateSource<C> {

    /// define and write metrics from aggregated scores to the target channel
    fn publish(&self) {
        self.target.write(|scope| {
            self.scores.for_each(|metric| {
                match metric.read_and_reset() {
                    AggregateScore::Event {hit} => {
                        let name = format!("{}.hit", &metric.name);
                        let temp_metric = self.target.define(MetricType::Count, name, 1.0);
                        scope.write(&temp_metric, hit);
                    },
                    AggregateScore::Value {hit, sum, max, min} => {
                        if hit > 0 {
                            // do not report gauges sum and hit, they are meaningless
                            match &metric.m_type {
                                &MetricType::Gauge => {
                                    // NOTE averaging badly
                                    // - hit and sum are not incremented nor read as one
                                    // - integer division is not rounding
                                    // assuming values will still be good enough to be useful
                                    let name = format!("{}.avg", &metric.name);
                                    let temp_metric = self.target.define(metric.m_type, name, 1.0);
                                    scope.write(&temp_metric, sum / hit);
                                },
                                &MetricType::Count | &MetricType::Time => {
                                    let name = format!("{}.sum", &metric.name);
                                    let temp_metric = self.target.define(metric.m_type, name, 1.0);
                                    scope.write(&temp_metric, sum);
                                },
                                _ => ()
                            }

                            let name = format!("{}.max", &metric.name);
                            let temp_metric = self.target.define(MetricType::Gauge, name, 1.0);
                            scope.write(&temp_metric, max);

                            let name = format!("{}.min", &metric.name);
                            let temp_metric = self.target.define(MetricType::Gauge, name, 1.0);
                            scope.write(&temp_metric, min);
                        }
                    }
                }
            });
        });

    }

}