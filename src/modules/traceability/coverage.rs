use crate::error::QmsResult;
use crate::modules::traceability::links::{TraceabilityLink, TraceabilityManager};
use crate::modules::traceability::requirement::Requirement;
use crate::modules::traceability::test_case::TestCase;

/// Coverage analysis types
#[derive(Debug, Clone, Copy)]
pub enum CoverageType {
    Requirements,
    Tests,
    Risks,
}

/// Coverage metrics for requirements
#[derive(Debug, Clone)]
pub struct RequirementsCoverage {
    pub total_requirements: usize,
    pub tested_requirements: usize,
    pub verified_requirements: usize,
    pub untested_requirements: Vec<String>,
    pub unverified_requirements: Vec<String>,
    pub coverage_percentage: f64,
    pub verification_percentage: f64,
}

/// Coverage metrics for tests
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TestsCoverage {
    pub total_tests: usize,
    pub linked_tests: usize,
    pub unlinked_tests: Vec<String>,
    pub coverage_percentage: f64,
    pub orphaned_tests: Vec<String>,
}

/// Coverage metrics for risks
#[derive(Debug, Clone)]
pub struct RisksCoverage {
    pub total_risks: usize,
    pub mitigated_risks: usize,
    pub unmitigated_risks: Vec<String>,
    pub coverage_percentage: f64,
}

/// Comprehensive coverage analysis
#[derive(Debug, Clone)]
pub struct CoverageAnalysis {
    pub requirements: RequirementsCoverage,
    pub tests: TestsCoverage,
    pub risks: RisksCoverage,
    pub overall_score: f64,
    pub gap_analysis: GapAnalysis,
}

/// Gap analysis results
#[derive(Debug, Clone)]
pub struct GapAnalysis {
    pub critical_gaps: Vec<String>,
    pub recommendations: Vec<String>,
    pub improvement_areas: Vec<String>,
}

/// Coverage analyzer for traceability analysis
pub struct CoverageAnalyzer {
    _traceability_manager: TraceabilityManager,
}

impl CoverageAnalyzer {
    /// Create a new coverage analyzer
    pub const fn new(traceability_manager: TraceabilityManager) -> Self {
        Self {
            _traceability_manager: traceability_manager,
        }
    }

    /// Analyze coverage for specific type
    pub fn analyze_coverage(&self, _coverage_type: CoverageType) -> QmsResult<CoverageAnalysis> {
        let requirements_coverage = self.analyze_requirements_coverage()?;
        let tests_coverage = self.analyze_tests_coverage()?;
        let risks_coverage = self.analyze_risks_coverage()?;
        
        let overall_score = self.calculate_overall_score(
            &requirements_coverage,
            &tests_coverage,
            &risks_coverage,
        );
        
        let gap_analysis = self.perform_gap_analysis(
            &requirements_coverage,
            &tests_coverage,
            &risks_coverage,
        );

        Ok(CoverageAnalysis {
            requirements: requirements_coverage,
            tests: tests_coverage,
            risks: risks_coverage,
            overall_score,
            gap_analysis,
        })
    }

    /// Analyze requirements coverage
    fn analyze_requirements_coverage(&self) -> QmsResult<RequirementsCoverage> {
        // Placeholder implementation - would integrate with actual requirement/test managers
        let requirements: Vec<Requirement> = vec![];
        let links: Vec<TraceabilityLink> = vec![];
        
        let total_requirements = requirements.len();
        let mut tested_requirements = 0;
        let mut verified_requirements = 0;
        let mut untested_requirements = Vec::new();
        let mut unverified_requirements = Vec::new();
        
        // Analyze each requirement
        for req in &requirements {
            let has_test_link = links.iter().any(|link| {
                link.source_id == req.id && link.target_type.to_string().contains("test")
            });
            
            let has_verification = links.iter().any(|link| {
                link.source_id == req.id && link.target_type.to_string().contains("verification")
            });
            
            if has_test_link {
                tested_requirements += 1;
            } else {
                untested_requirements.push(req.id.clone());
            }
            
            if has_verification {
                verified_requirements += 1;
            } else {
                unverified_requirements.push(req.id.clone());
            }
        }
        
        let coverage_percentage = if total_requirements > 0 {
            (tested_requirements as f64 / total_requirements as f64) * 100.0
        } else {
            0.0
        };
        
        let verification_percentage = if total_requirements > 0 {
            (verified_requirements as f64 / total_requirements as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(RequirementsCoverage {
            total_requirements,
            tested_requirements,
            verified_requirements,
            untested_requirements,
            unverified_requirements,
            coverage_percentage,
            verification_percentage,
        })
    }

    /// Analyze tests coverage
    fn analyze_tests_coverage(&self) -> QmsResult<TestsCoverage> {
        // Placeholder implementation - would integrate with actual test manager
        let test_cases: Vec<TestCase> = vec![];
        let links: Vec<TraceabilityLink> = vec![];
        
        let total_tests = test_cases.len();
        let mut linked_tests = 0;
        let mut unlinked_tests = Vec::new();
        let mut orphaned_tests = Vec::new();
        
        // Analyze each test case
        for test in &test_cases {
            let has_requirement_link = links.iter().any(|link| {
                link.source_id == test.test_id && link.target_type.to_string().contains("requirement")
            });
            
            if has_requirement_link {
                linked_tests += 1;
            } else {
                unlinked_tests.push(test.test_id.clone());
                orphaned_tests.push(format!("{}: {}", test.test_id, test.title));
            }
        }
        
        let coverage_percentage = if total_tests > 0 {
            (linked_tests as f64 / total_tests as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(TestsCoverage {
            total_tests,
            linked_tests,
            unlinked_tests,
            coverage_percentage,
            orphaned_tests,
        })
    }

    /// Analyze risks coverage  
    fn analyze_risks_coverage(&self) -> QmsResult<RisksCoverage> {
        // Placeholder implementation - would integrate with actual risk manager
        let total_risks = 0;
        let mitigated_risks = 0;
        let unmitigated_risks = Vec::new();
        
        let coverage_percentage = if total_risks > 0 {
            (mitigated_risks as f64 / total_risks as f64) * 100.0
        } else {
            0.0
        };
        
        Ok(RisksCoverage {
            total_risks,
            mitigated_risks,
            unmitigated_risks,
            coverage_percentage,
        })
    }

    /// Calculate overall coverage score
    fn calculate_overall_score(
        &self,
        requirements: &RequirementsCoverage,
        tests: &TestsCoverage,
        risks: &RisksCoverage,
    ) -> f64 {
        // Weighted average of coverage metrics
        let weights = (0.4, 0.3, 0.3); // Requirements, Tests, Risks
        
        requirements.coverage_percentage * weights.0 +
         tests.coverage_percentage * weights.1 +
         risks.coverage_percentage * weights.2
    }

    /// Perform gap analysis
    fn perform_gap_analysis(
        &self,
        requirements: &RequirementsCoverage,
        tests: &TestsCoverage,
        risks: &RisksCoverage,
    ) -> GapAnalysis {
        let mut critical_gaps = Vec::new();
        let mut recommendations = Vec::new();
        let mut improvement_areas = Vec::new();
        
        // Identify critical gaps
        if requirements.coverage_percentage < 80.0 {
            critical_gaps.push(format!(
                "Low requirements coverage: {:.1}% (target: 80%)",
                requirements.coverage_percentage
            ));
        }
        
        if tests.coverage_percentage < 90.0 {
            critical_gaps.push(format!(
                "Low test linkage: {:.1}% (target: 90%)",
                tests.coverage_percentage
            ));
        }
        
        if risks.coverage_percentage < 85.0 {
            critical_gaps.push(format!(
                "Low risk mitigation: {:.1}% (target: 85%)",
                risks.coverage_percentage
            ));
        }
        
        // Generate recommendations
        if !requirements.untested_requirements.is_empty() {
            recommendations.push(format!(
                "Create test cases for {} untested requirements",
                requirements.untested_requirements.len()
            ));
        }
        
        if !tests.orphaned_tests.is_empty() {
            recommendations.push(format!(
                "Link {} orphaned test cases to requirements",
                tests.orphaned_tests.len()
            ));
        }
        
        if !requirements.unverified_requirements.is_empty() {
            recommendations.push(format!(
                "Add verification evidence for {} requirements",
                requirements.unverified_requirements.len()
            ));
        }
        
        // Identify improvement areas
        if requirements.coverage_percentage < 95.0 {
            improvement_areas.push("Requirements test coverage".to_string());
        }
        
        if requirements.verification_percentage < 90.0 {
            improvement_areas.push("Requirements verification".to_string());
        }
        
        if tests.coverage_percentage < 95.0 {
            improvement_areas.push("Test case linkage".to_string());
        }
        
        GapAnalysis {
            critical_gaps,
            recommendations,
            improvement_areas,
        }
    }

    /// Generate coverage dashboard
    pub fn generate_dashboard(&self, analysis: &CoverageAnalysis) -> String {
        let mut dashboard = String::new();
        
        dashboard.push_str("QMS TRACEABILITY COVERAGE DASHBOARD\n");
        dashboard.push_str("═══════════════════════════════════════\n\n");
        
        // Overall score
        dashboard.push_str(&format!(
            "Overall Coverage Score: {:.1}%\n",
            analysis.overall_score
        ));
        
        // Status indicator
        let status = if analysis.overall_score >= 90.0 {
            "✅ EXCELLENT"
        } else if analysis.overall_score >= 80.0 {
            "✅ GOOD"
        } else if analysis.overall_score >= 70.0 {
            "⚠️ NEEDS IMPROVEMENT"
        } else {
            "❌ CRITICAL"
        };
        
        dashboard.push_str(&format!("Status: {status}\n\n"));
        
        // Requirements coverage
        dashboard.push_str("REQUIREMENTS COVERAGE\n");
        dashboard.push_str("─────────────────────\n");
        dashboard.push_str(&format!(
            "Total Requirements: {}\n",
            analysis.requirements.total_requirements
        ));
        dashboard.push_str(&format!(
            "Tested: {} ({:.1}%)\n",
            analysis.requirements.tested_requirements,
            analysis.requirements.coverage_percentage
        ));
        dashboard.push_str(&format!(
            "Verified: {} ({:.1}%)\n",
            analysis.requirements.verified_requirements,
            analysis.requirements.verification_percentage
        ));
        dashboard.push_str(&format!(
            "Untested: {}\n\n",
            analysis.requirements.untested_requirements.len()
        ));
        
        // Tests coverage
        dashboard.push_str("TESTS COVERAGE\n");
        dashboard.push_str("──────────────\n");
        dashboard.push_str(&format!(
            "Total Tests: {}\n",
            analysis.tests.total_tests
        ));
        dashboard.push_str(&format!(
            "Linked: {} ({:.1}%)\n",
            analysis.tests.linked_tests,
            analysis.tests.coverage_percentage
        ));
        dashboard.push_str(&format!(
            "Orphaned: {}\n\n",
            analysis.tests.orphaned_tests.len()
        ));
        
        // Risks coverage
        dashboard.push_str("RISKS COVERAGE\n");
        dashboard.push_str("──────────────\n");
        dashboard.push_str(&format!(
            "Total Risks: {}\n",
            analysis.risks.total_risks
        ));
        dashboard.push_str(&format!(
            "Mitigated: {} ({:.1}%)\n",
            analysis.risks.mitigated_risks,
            analysis.risks.coverage_percentage
        ));
        dashboard.push_str(&format!(
            "Unmitigated: {}\n\n",
            analysis.risks.unmitigated_risks.len()
        ));
        
        // Gap analysis
        dashboard.push_str("GAP ANALYSIS\n");
        dashboard.push_str("────────────\n");
        
        if analysis.gap_analysis.critical_gaps.is_empty() {
            dashboard.push_str("✅ No critical gaps identified\n");
        } else {
            dashboard.push_str("Critical Gaps:\n");
            for gap in &analysis.gap_analysis.critical_gaps {
                dashboard.push_str(&format!("  ❌ {gap}\n"));
            }
        }
        
        dashboard.push_str("\nRecommendations:\n");
        for rec in &analysis.gap_analysis.recommendations {
            dashboard.push_str(&format!("  💡 {rec}\n"));
        }
        
        dashboard.push_str("\nImprovement Areas:\n");
        for area in &analysis.gap_analysis.improvement_areas {
            dashboard.push_str(&format!("  📈 {area}\n"));
        }
        
        dashboard
    }

    /// Generate coverage report
    pub fn generate_report(&self, analysis: &CoverageAnalysis, format: &str) -> QmsResult<String> {
        match format {
            "text" => Ok(self.generate_dashboard(analysis)),
            "csv" => Ok(self.generate_csv_report(analysis)),
            "json" => Ok(self.generate_json_report(analysis)),
            _ => Err(crate::error::QmsError::validation_error(
                &format!("Unsupported format: {format}")
            )),
        }
    }

    /// Generate CSV report
    fn generate_csv_report(&self, analysis: &CoverageAnalysis) -> String {
        let mut csv = String::new();
        
        csv.push_str("Metric,Value,Percentage\n");
        csv.push_str(&format!(
            "Overall Score,{:.1}%,{:.1}\n",
            analysis.overall_score,
            analysis.overall_score
        ));
        csv.push_str(&format!(
            "Requirements Coverage,{}/{},{:.1}\n",
            analysis.requirements.tested_requirements,
            analysis.requirements.total_requirements,
            analysis.requirements.coverage_percentage
        ));
        csv.push_str(&format!(
            "Test Coverage,{}/{},{:.1}\n",
            analysis.tests.linked_tests,
            analysis.tests.total_tests,
            analysis.tests.coverage_percentage
        ));
        csv.push_str(&format!(
            "Risk Coverage,{}/{},{:.1}\n",
            analysis.risks.mitigated_risks,
            analysis.risks.total_risks,
            analysis.risks.coverage_percentage
        ));
        
        csv
    }

    /// Generate JSON report
    fn generate_json_report(&self, analysis: &CoverageAnalysis) -> String {
        // Simplified JSON output (would use serde in real implementation)
        format!(
            r#"{{
  "overall_score": {:.1},
  "requirements": {{
    "total": {},
    "tested": {},
    "verified": {},
    "coverage_percentage": {:.1},
    "verification_percentage": {:.1}
  }},
  "tests": {{
    "total": {},
    "linked": {},
    "coverage_percentage": {:.1}
  }},
  "risks": {{
    "total": {},
    "mitigated": {},
    "coverage_percentage": {:.1}
  }},
  "gap_analysis": {{
    "critical_gaps": {},
    "recommendations": {},
    "improvement_areas": {}
  }}
}}"#,
            analysis.overall_score,
            analysis.requirements.total_requirements,
            analysis.requirements.tested_requirements,
            analysis.requirements.verified_requirements,
            analysis.requirements.coverage_percentage,
            analysis.requirements.verification_percentage,
            analysis.tests.total_tests,
            analysis.tests.linked_tests,
            analysis.tests.coverage_percentage,
            analysis.risks.total_risks,
            analysis.risks.mitigated_risks,
            analysis.risks.coverage_percentage,
            analysis.gap_analysis.critical_gaps.len(),
            analysis.gap_analysis.recommendations.len(),
            analysis.gap_analysis.improvement_areas.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coverage_analysis() {
        // Test coverage analysis creation
        let project_root = std::env::current_dir().unwrap();
        let traceability_manager = TraceabilityManager::new(&project_root).unwrap();
        let analyzer = CoverageAnalyzer::new(traceability_manager);
        
        // Test coverage analysis
        let result = analyzer.analyze_coverage(CoverageType::Requirements);
        assert!(result.is_ok());
        
        let analysis = result.unwrap();
        assert!(analysis.overall_score >= 0.0);
        assert!(analysis.overall_score <= 100.0);
    }

    #[test]
    fn test_dashboard_generation() {
        let project_root = std::env::current_dir().unwrap();
        let traceability_manager = TraceabilityManager::new(&project_root).unwrap();
        let analyzer = CoverageAnalyzer::new(traceability_manager);
        
        let analysis = analyzer.analyze_coverage(CoverageType::Requirements).unwrap();
        let dashboard = analyzer.generate_dashboard(&analysis);
        
        assert!(dashboard.contains("QMS TRACEABILITY COVERAGE DASHBOARD"));
        assert!(dashboard.contains("Overall Coverage Score"));
        assert!(dashboard.contains("REQUIREMENTS COVERAGE"));
        assert!(dashboard.contains("TESTS COVERAGE"));
        assert!(dashboard.contains("RISKS COVERAGE"));
        assert!(dashboard.contains("GAP ANALYSIS"));
    }

    #[test]
    fn test_report_formats() {
        let project_root = std::env::current_dir().unwrap();
        let traceability_manager = TraceabilityManager::new(&project_root).unwrap();
        let analyzer = CoverageAnalyzer::new(traceability_manager);
        
        let analysis = analyzer.analyze_coverage(CoverageType::Requirements).unwrap();
        
        // Test text format
        let text_report = analyzer.generate_report(&analysis, "text").unwrap();
        assert!(text_report.contains("Overall Coverage Score"));
        
        // Test CSV format
        let csv_report = analyzer.generate_report(&analysis, "csv").unwrap();
        assert!(csv_report.contains("Metric,Value,Percentage"));
        
        // Test JSON format
        let json_report = analyzer.generate_report(&analysis, "json").unwrap();
        assert!(json_report.contains("overall_score"));
        
        // Test invalid format
        let invalid_result = analyzer.generate_report(&analysis, "invalid");
        assert!(invalid_result.is_err());
    }

    #[test]
    fn test_gap_analysis() {
        let requirements = RequirementsCoverage {
            total_requirements: 10,
            tested_requirements: 7,
            verified_requirements: 6,
            untested_requirements: vec!["REQ-001".to_string(), "REQ-002".to_string()],
            unverified_requirements: vec!["REQ-003".to_string()],
            coverage_percentage: 70.0,
            verification_percentage: 60.0,
        };
        
        let tests = TestsCoverage {
            total_tests: 15,
            linked_tests: 12,
            unlinked_tests: vec!["TC-001".to_string()],
            coverage_percentage: 80.0,
            orphaned_tests: vec!["TC-001: Orphaned Test".to_string()],
        };
        
        let risks = RisksCoverage {
            total_risks: 5,
            mitigated_risks: 3,
            unmitigated_risks: vec!["RISK-001".to_string()],
            coverage_percentage: 60.0,
        };
        
        let traceability_manager = TraceabilityManager::new(&std::env::current_dir().unwrap()).unwrap();
        let analyzer = CoverageAnalyzer::new(traceability_manager);
        
        let gap_analysis = analyzer.perform_gap_analysis(&requirements, &tests, &risks);
        
        assert!(!gap_analysis.critical_gaps.is_empty());
        assert!(!gap_analysis.recommendations.is_empty());
        assert!(!gap_analysis.improvement_areas.is_empty());
    }

    #[test]
    fn test_overall_score_calculation() {
        let requirements = RequirementsCoverage {
            total_requirements: 10,
            tested_requirements: 8,
            verified_requirements: 7,
            untested_requirements: vec![],
            unverified_requirements: vec![],
            coverage_percentage: 80.0,
            verification_percentage: 70.0,
        };
        
        let tests = TestsCoverage {
            total_tests: 10,
            linked_tests: 9,
            unlinked_tests: vec![],
            coverage_percentage: 90.0,
            orphaned_tests: vec![],
        };
        
        let risks = RisksCoverage {
            total_risks: 5,
            mitigated_risks: 4,
            unmitigated_risks: vec![],
            coverage_percentage: 80.0,
        };
        
        let traceability_manager = TraceabilityManager::new(&std::env::current_dir().unwrap()).unwrap();
        let analyzer = CoverageAnalyzer::new(traceability_manager);
        
        let overall_score = analyzer.calculate_overall_score(&requirements, &tests, &risks);
        
        // Expected: 80.0 * 0.4 + 90.0 * 0.3 + 80.0 * 0.3 = 32.0 + 27.0 + 24.0 = 83.0
        assert_eq!(overall_score, 83.0);
    }
}