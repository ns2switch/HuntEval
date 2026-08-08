use std::fmt::Write;

use super::{BenchmarkResult, BenchmarkResultError};

#[derive(Debug, Default, Clone, Copy)]
pub struct BenchmarkStaticHtmlRenderer;

impl BenchmarkStaticHtmlRenderer {
    pub fn render(&self, report: &BenchmarkResult) -> Result<Vec<u8>, BenchmarkResultError> {
        report.validate()?;
        let mut deployments = String::new();
        for summary in &report.deployments {
            write!(
                deployments,
                "<tr><th scope=\"row\">{}</th><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                escape(summary.deployment_id.as_str()),
                summary.completed_cells,
                summary.failed_cells,
                summary.disqualifying_constraints,
                score(summary.aggregate_score.mean)
            )
            .map_err(|_| BenchmarkResultError::InvalidContract)?;
        }
        let mut comparisons = String::new();
        for comparison in &report.comparisons {
            write!(
                comparisons,
                "<article id=\"{}\"><h3>{} versus {}</h3><p>Eligibility: {}. Paired samples: {}. Conclusion: {}.</p></article>",
                escape(&comparison.comparison_id),
                escape(comparison.left.as_str()),
                escape(comparison.right.as_str()),
                if comparison.eligible { "eligible" } else { "ineligible" },
                comparison.aggregate_difference.count,
                if comparison.aggregate_difference.conclusive { "conclusive" } else { "inconclusive" }
            )
            .map_err(|_| BenchmarkResultError::InvalidContract)?;
        }
        let rankings = render_rankings(report)?;
        let metrics = render_metrics(report)?;
        let cells = render_cells(report)?;
        let claims = render_claims(report)?;
        let artifacts = render_artifacts(report)?;
        let mut timelines = String::new();
        for cell in &report.cells {
            for entry in &cell.submitted_timeline {
                write!(
                    timelines,
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    escape(&cell.cell_id.to_string()),
                    escape(cell.deployment_id.as_str()),
                    escape(&entry.observed_at.to_string()),
                    escape(&entry.summary)
                )
                .map_err(|_| BenchmarkResultError::InvalidContract)?;
            }
        }
        let limitations = report
            .limitations
            .iter()
            .map(|item| format!("<li>{}</li>", escape(item)))
            .collect::<String>();
        let document = format!(
            "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width,initial-scale=1\"><title>HuntEval benchmark report</title><style>body{{font:16px system-ui;max-width:1100px;margin:auto;padding:1rem}}table{{border-collapse:collapse;width:100%}}th,td{{border:1px solid #888;padding:.4rem;text-align:left;vertical-align:top}}code{{overflow-wrap:anywhere}}.muted{{color:#555}}</style></head><body><header><h1>Benchmark {}</h1><p class=\"muted\">Static, script-free report</p></header><main><section aria-labelledby=\"overview\"><h2 id=\"overview\">Overview</h2><table><thead><tr><th>Deployment</th><th>Completed</th><th>Failed</th><th>Disqualifying constraints</th><th>Mean score</th></tr></thead><tbody>{deployments}</tbody></table></section><section aria-labelledby=\"rankings\"><h2 id=\"rankings\">Constraint-first ranking</h2>{rankings}</section><section aria-labelledby=\"metrics\"><h2 id=\"metrics\">Metric summaries and provenance</h2>{metrics}</section><section aria-labelledby=\"comparisons\"><h2 id=\"comparisons\">Comparisons</h2>{comparisons}</section><section aria-labelledby=\"cells\"><h2 id=\"cells\">Cell inventory and constraints</h2>{cells}</section><section aria-labelledby=\"timeline\"><h2 id=\"timeline\">Submitted timelines and attribution</h2><table><thead><tr><th>Cell</th><th>Deployment</th><th>Observed at</th><th>Submitted summary</th></tr></thead><tbody>{timelines}</tbody></table><p>Attribution is observational and does not establish causality.</p></section><section aria-labelledby=\"claims\"><h2 id=\"claims\">Evidence-linked claims</h2>{claims}</section><section aria-labelledby=\"artifacts\"><h2 id=\"artifacts\">Verified artifacts</h2>{artifacts}</section><section aria-labelledby=\"limitations\"><h2 id=\"limitations\">Limitations</h2><ul>{limitations}</ul></section></main></body></html>\n",
            escape(report.benchmark_id.as_str())
        );
        Ok(document.into_bytes())
    }
}

fn render_rankings(report: &BenchmarkResult) -> Result<String, BenchmarkResultError> {
    let mut output = String::from("<ol>");
    for group in &report.rankings {
        let deployments = group
            .deployments
            .iter()
            .map(|item| escape(item.as_str()))
            .collect::<Vec<_>>()
            .join(", ");
        write!(
            output,
            "<li>Rank {}: {} — score {}; disqualifying constraints: {}</li>",
            group.rank,
            deployments,
            score(group.aggregate_score),
            group.disqualifying_constraints
        )
        .map_err(|_| BenchmarkResultError::InvalidContract)?;
    }
    output.push_str("</ol>");
    Ok(output)
}

fn render_metrics(report: &BenchmarkResult) -> Result<String, BenchmarkResultError> {
    let mut output = String::new();
    for deployment in &report.deployments {
        write!(
            output,
            "<h3>{}</h3><ul>",
            escape(deployment.deployment_id.as_str())
        )
        .map_err(|_| BenchmarkResultError::InvalidContract)?;
        for metric in deployment.metrics.values() {
            write!(
                output,
                "<li>{}: mean {}; samples {}</li>",
                escape(&metric.metric),
                score(metric.statistics.mean),
                metric.statistics.count
            )
            .map_err(|_| BenchmarkResultError::InvalidContract)?;
        }
        output.push_str("</ul>");
    }
    Ok(output)
}

fn render_cells(report: &BenchmarkResult) -> Result<String, BenchmarkResultError> {
    let mut output = String::from(
        "<table><thead><tr><th>Cell</th><th>Status</th><th>Score</th><th>Constraints and omissions</th></tr></thead><tbody>",
    );
    for cell in &report.cells {
        let constraints = cell
            .constraints
            .iter()
            .map(|item| format!("{}: {}", escape(&item.code), escape(&item.status)))
            .chain(
                cell.aggregate_score_omissions
                    .iter()
                    .map(|(metric, reason)| {
                        format!("{} omitted: {}", escape(metric), escape(reason))
                    }),
            )
            .collect::<Vec<_>>()
            .join("; ");
        write!(
            output,
            "<tr><th scope=\"row\"><code>{}</code></th><td>{}</td><td>{}</td><td>{}</td></tr>",
            escape(&cell.cell_id.to_string()),
            escape(&cell.status),
            score(cell.aggregate_score),
            constraints
        )
        .map_err(|_| BenchmarkResultError::InvalidContract)?;
    }
    output.push_str("</tbody></table>");
    Ok(output)
}

fn render_claims(report: &BenchmarkResult) -> Result<String, BenchmarkResultError> {
    let mut output = String::from("<ul>");
    for claim in &report.claims {
        write!(
            output,
            "<li><strong>{}</strong>: {} ({} validated source(s))</li>",
            escape(&claim.claim_id),
            escape(&claim.text),
            claim.sources.len()
        )
        .map_err(|_| BenchmarkResultError::InvalidContract)?;
    }
    output.push_str("</ul>");
    Ok(output)
}

fn render_artifacts(report: &BenchmarkResult) -> Result<String, BenchmarkResultError> {
    let mut output = String::from("<ul>");
    for artifact in &report.artifacts {
        write!(
            output,
            "<li><code>{}</code> — SHA-256 <code>{}</code></li>",
            escape(&artifact.path),
            artifact.sha256
        )
        .map_err(|_| BenchmarkResultError::InvalidContract)?;
    }
    output.push_str("</ul>");
    Ok(output)
}

fn score(value: Option<f64>) -> String {
    value.map_or_else(|| "not available".into(), |value| format!("{value:.4}"))
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
