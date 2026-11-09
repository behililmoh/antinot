/// Module de calcul et d'évaluation mathématique inline pour Antinot.
/// Permet d'évaluer des expressions mathématiques simples et avancées.

pub struct Calculator;

impl Calculator {
    /// Évalue une expression mathématique textuelle (ex: "15 * 4.5", "sqrt(144) + 10", "100 * 1.20").
    /// Retourne le résultat formaté en chaîne ou None si l'expression est invalide.
    pub fn evaluate(expr: &str) -> Option<String> {
        let clean = expr.trim();
        if clean.is_empty() {
            return None;
        }

        // Nettoyage des virgules françaises en points pour les nombres décimaux
        let sanitized = sanitize_expression(clean);
    let sanitized = sanitized.replace("sqrt(", "math::sqrt(");

        // Évaluation via evalexpr
        match evalexpr::eval(&sanitized) {
            Ok(value) => match value {
                evalexpr::Value::Float(f) => {
                    if f.is_nan() || f.is_infinite() {
                        None
                    } else if (f.fract()).abs() < 1e-9 {
                        Some(format!("{:.0}", f))
                    } else {
                        // Supprime les zéros inutiles à la fin
                        let s = format!("{:.6}", f);
                        let s = s.trim_end_matches('0').trim_end_matches('.');
                        Some(s.to_string())
                    }
                }
                evalexpr::Value::Int(i) => Some(i.to_string()),
                evalexpr::Value::Boolean(b) => Some(b.to_string()),
                _ => None,
            },
            Err(_) => None,
        }
    }

    /// Détecte si la ligne se termine par un signe `=` ou `= ` avec une expression mathématique avant.
    /// Ex: "15 * 4 =" -> retourne Some((prefix_sans_egal, "60"))
    pub fn try_eval_equation(line: &str) -> Option<(&str, String)> {
        let trimmed = line.trim_end();
        if trimmed.ends_with('=') {
            let expr = trimmed[..trimmed.len() - 1].trim();
            if let Some(res) = Self::evaluate(expr) {
                return Some((expr, res));
            }
        }
        None
    }

    /// Analyse la ligne courante pour voir si elle constitue ou contient une expression mathématique valide.
    pub fn analyze_line(line: &str) -> Option<String> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return None;
        }

        // Si la ligne se termine par '=', évaluer ce qui précède
        if let Some((_, res)) = Self::try_eval_equation(trimmed) {
            return Some(res);
        }

        // Sinon, tester si la ligne entière est une expression arithmétique
        // Pour éviter de faux positifs sur de simples mots, on vérifie la présence d'au moins un opérateur ou chiffre
        let has_math_char = trimmed.chars().any(|c| "+-*/%^".contains(c));
        let has_digit = trimmed.chars().any(|c| c.is_ascii_digit());

        if has_math_char && has_digit {
            if let Some(res) = Self::evaluate(trimmed) {
                // Éviter de retourner le résultat s'il est identique à l'expression (ex: juste "42")
                if res != trimmed {
                    return Some(res);
                }
            }
        }

        None
    }
}

/// Prépare l'expression pour l'évaluateur (ex: remplace % par /100 si applicable)
fn sanitize_expression(expr: &str) -> String {
    let mut out = String::with_capacity(expr.len() + 8);
    let chars: Vec<char> = expr.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let c = chars[i];
        if c == ',' {
            // Remplace la virgule par un point s'il est entouré de chiffres
            let prev_is_digit = i > 0 && chars[i - 1].is_ascii_digit();
            let next_is_digit = i + 1 < len && chars[i + 1].is_ascii_digit();
            if prev_is_digit && next_is_digit {
                out.push('.');
            } else {
                out.push(',');
            }
        } else if c == '%' {
            out.push_str("/ 100.0");
        } else if c == '/' {
            // Force la division flottante dans evalexpr (évite que 10 / 4 devienne 2 au lieu de 2.5)
            out.push_str("/ 1.0 /");
        } else {
            out.push(c);
        }
        i += 1;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_simple() {
        assert_eq!(Calculator::evaluate("12 + 8"), Some("20".to_string()));
        assert_eq!(Calculator::evaluate("15 * 4"), Some("60".to_string()));
        assert_eq!(Calculator::evaluate("10 / 4"), Some("2.5".to_string()));
        assert_eq!(Calculator::evaluate("sqrt(16)"), Some("4".to_string()));
    }

    #[test]
    fn test_eval_equation() {
        assert_eq!(
            Calculator::try_eval_equation("25 * 4 ="),
            Some(("25 * 4", "100".to_string()))
        );
    }
}
