use colored::Colorize;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

const STATS_FILE: &str = "game_stats.json";

#[derive(Debug, Clone, Copy, PartialEq)]
enum Move {
    Cooperate,
    Defect,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Difficulty {
    Easy,
    Medium,
    Hard,
    Legendary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Statistics {
    games_played: u32,
    games_won: u32,
    games_lost: u32,
    games_tied: u32,
    total_points: i32,
    best_score_differential: i32,
    worst_score_differential: i32,
}

impl Statistics {
    fn new() -> Self {
        Statistics {
            games_played: 0,
            games_won: 0,
            games_lost: 0,
            games_tied: 0,
            total_points: 0,
            best_score_differential: 0,
            worst_score_differential: 0,
        }
    }

    fn load() -> Self {
        if Path::new(STATS_FILE).exists() {
            if let Ok(content) = fs::read_to_string(STATS_FILE) {
                if let Ok(stats) = serde_json::from_str(&content) {
                    return stats;
                }
            }
        }
        Statistics::new()
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string_pretty(self) {
            let _ = fs::write(STATS_FILE, json);
        }
    }

    fn win_rate(&self) -> f32 {
        if self.games_played == 0 {
            0.0
        } else {
            (self.games_won as f32 / self.games_played as f32) * 100.0
        }
    }
}

#[derive(Debug)]
struct GameState {
    player_score: i32,
    computer_score: i32,
    round: u32,
    total_rounds: u32,
    history: Vec<(Move, Move)>,
    difficulty: Difficulty,
}

impl GameState {
    fn new(total_rounds: u32, difficulty: Difficulty) -> Self {
        GameState {
            player_score: 0,
            computer_score: 0,
            round: 0,
            total_rounds,
            history: Vec::new(),
            difficulty,
        }
    }

    fn calculate_payoff(&self, player_move: Move, computer_move: Move) -> (i32, i32) {
        match (player_move, computer_move) {
            (Move::Cooperate, Move::Cooperate) => (3, 3),
            (Move::Cooperate, Move::Defect) => (0, 5),
            (Move::Defect, Move::Cooperate) => (5, 0),
            (Move::Defect, Move::Defect) => (1, 1),
        }
    }

    fn game_progress_bar(&self) -> String {
        let filled = (self.round as f32 / self.total_rounds as f32 * 30.0) as usize;
        let empty = 30 - filled;
        let bar = format!(
            "{}{}",
            "█".repeat(filled).green(),
            "░".repeat(empty).dimmed()
        );
        format!(
            "[{}] {}/{}",
            bar,
            self.round.to_string().cyan(),
            self.total_rounds.to_string().cyan()
        )
    }
}

fn clear_screen() {
    print!("\x1B[2J\x1B[1;1H");
    io::stdout().flush().unwrap();
}

fn print_title() {
    clear_screen();
    println!("{}", "╔═══════════════════════════════════════════════════════════╗".bright_cyan());
    println!(
        "{}",
        "║                                                           ║".bright_cyan()
    );
    println!(
        "{}",
        format!("║  {}  ║", "[*] GAME THEORY: PRISONER'S DILEMMA [*]".bold())
            .bright_cyan()
    );
    println!(
        "{}",
        format!("║  {}  ║", "Terminal Edition - Strategic Gameplay".italic())
            .bright_cyan()
    );
    println!(
        "{}",
        "║                                                           ║".bright_cyan()
    );
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".bright_cyan());
    println!();
}

fn print_payoff_matrix() {
    println!("{}", "═".repeat(60).bright_black());
    println!("{}", "PAYOFF MATRIX (Your Points / Computer Points)".yellow().bold());
    println!("{}", "═".repeat(60).bright_black());
    println!(
        "  {} {}",
        "[C] Both Cooperate:".green(),
        "3 / 3 (Mutual Benefit)".bright_green()
    );
    println!(
        "  {} {}",
        "[+] You Cooperate, Opponent Defects:".yellow(),
        "0 / 5 (Sucker's Payoff)".bright_yellow()
    );
    println!(
        "  {} {}",
        "[-] You Defect, Opponent Cooperates:".red(),
        "5 / 0 (Temptation Payoff)".bright_red()
    );
    println!(
        "  {} {}",
        "[X] Both Defect:".magenta(),
        "1 / 1 (Mutual Punishment)".bright_magenta()
    );
    println!("{}", "═".repeat(60).bright_black());
    println!();
}

fn print_difficulty_menu() -> Difficulty {
    println!("{}", "Choose Difficulty Level:".yellow().bold());
    println!();
    println!("  {} - Computer cooperates 70% of the time", "[1] EASY".green().bold());
    println!(
        "  {} - Computer uses pure tit-for-tat",
        "[2] MEDIUM".yellow().bold()
    );
    println!(
        "  {} - Computer defects strategically 40% of the time",
        "[3] HARD".red().bold()
    );
    println!("  {} - Computer is unpredictable and ruthless", "[4] LEGENDARY".magenta().bold());
    println!();

    loop {
        print!("{}: ", "Select difficulty (1-4)".cyan().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim() {
            "1" => return Difficulty::Easy,
            "2" => return Difficulty::Medium,
            "3" => return Difficulty::Hard,
            "4" => return Difficulty::Legendary,
            _ => {
                println!("{}", "Invalid choice! Please enter 1-4.".red());
            }
        }
    }
}

fn print_game_state(state: &GameState) {
    println!("\n{}", state.game_progress_bar());
    println!();

    let player_color = if state.player_score > state.computer_score {
        state.player_score.to_string().bright_green()
    } else if state.player_score < state.computer_score {
        state.player_score.to_string().bright_red()
    } else {
        state.player_score.to_string().yellow()
    };

    let computer_color = if state.computer_score > state.player_score {
        state.computer_score.to_string().bright_green()
    } else if state.computer_score < state.player_score {
        state.computer_score.to_string().bright_red()
    } else {
        state.computer_score.to_string().yellow()
    };

    println!(
        "  {} {} │ {} {}",
        "You:".cyan().bold(),
        player_color,
        "Computer:".magenta().bold(),
        computer_color
    );
    println!();
}

fn animate_choice(choice_text: &str, emoji: &str) {
    print!("  ");
    for c in choice_text.chars() {
        print!("{}", c.to_string().cyan());
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(20));
    }
    println!(" {}", emoji);
}

fn get_player_move() -> Move {
    println!();
    println!("{}", "Your Turn - Choose your strategy:".yellow().bold());
    println!();
    animate_choice("[1] COOPERATE", "[C]");
    println!("       Trust and work together for mutual benefit");
    println!();
    animate_choice("[2] DEFECT", "[D]");
    println!("       Act in self-interest and betray");
    println!();

    loop {
        print!("{}: ", "Your choice (1 or 2)".cyan().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim() {
            "1" => return Move::Cooperate,
            "2" => return Move::Defect,
            _ => {
                println!("{}", "[!] Invalid input. Please enter 1 or 2.".red());
            }
        }
    }
}

fn get_computer_move(history: &[(Move, Move)], difficulty: Difficulty) -> Move {
    let mut rng = rand::thread_rng();

    match difficulty {
        Difficulty::Easy => {
            if rng.gen_bool(0.7) {
                Move::Cooperate
            } else {
                Move::Defect
            }
        }
        Difficulty::Medium => {
            if history.is_empty() {
                Move::Cooperate
            } else {
                let last_player_move = history.last().unwrap().0;
                if rng.gen_bool(0.85) {
                    last_player_move
                } else {
                    Move::Defect
                }
            }
        }
        Difficulty::Hard => {
            if history.is_empty() {
                if rng.gen_bool(0.6) {
                    Move::Cooperate
                } else {
                    Move::Defect
                }
            } else {
                let player_defect_rate = history
                    .iter()
                    .filter(|(p, _)| *p == Move::Defect)
                    .count() as f32
                    / history.len() as f32;

                if player_defect_rate > 0.4 {
                    Move::Defect
                } else if rng.gen_bool(0.6) {
                    Move::Cooperate
                } else {
                    Move::Defect
                }
            }
        }
        Difficulty::Legendary => {
            if history.is_empty() {
                if rng.gen_bool(0.5) {
                    Move::Cooperate
                } else {
                    Move::Defect
                }
            } else {
                let player_defect_rate = history
                    .iter()
                    .filter(|(p, _)| *p == Move::Defect)
                    .count() as f32
                    / history.len() as f32;

                let last_move = history.last().unwrap().0;
                let mut strategy = if player_defect_rate > 0.3 {
                    Move::Defect
                } else if last_move == Move::Defect {
                    Move::Defect
                } else if rng.gen_bool(0.5) {
                    Move::Defect
                } else {
                    Move::Cooperate
                };

                if rng.gen_bool(0.15) {
                    strategy = if strategy == Move::Cooperate {
                        Move::Defect
                    } else {
                        Move::Cooperate
                    };
                }

                strategy
            }
        }
    }
}

fn animate_round_result(
    player_move: Move,
    computer_move: Move,
    player_points: i32,
    computer_points: i32,
) {
    thread::sleep(Duration::from_millis(800));

    let player_str = if player_move == Move::Cooperate {
        "[C] COOPERATE".green()
    } else {
        "[D] DEFECT".red()
    };

    let computer_str = if computer_move == Move::Cooperate {
        "[C] COOPERATE".green()
    } else {
        "[D] DEFECT".red()
    };

    println!("\n{}", "╔════════════════════════════════════════════╗".bright_cyan());
    println!(
        "{}",
        "║          ROUND RESOLUTION                   ║".bright_cyan()
    );
    println!("{}", "╠════════════════════════════════════════════╣".bright_cyan());

    println!(
        "{}",
        format!("║  You:     {}                      ║", player_str)
            .bright_cyan()
    );
    println!(
        "{}",
        format!("║  Computer: {}                    ║", computer_str)
            .bright_cyan()
    );

    println!("{}", "╠════════════════════════════════════════════╣".bright_cyan());

    let player_color = if player_points > computer_points {
        player_points.to_string().bright_green()
    } else if player_points < computer_points {
        player_points.to_string().bright_red()
    } else {
        player_points.to_string().yellow()
    };

    let computer_color = if computer_points > player_points {
        computer_points.to_string().bright_green()
    } else if computer_points < player_points {
        computer_points.to_string().bright_red()
    } else {
        computer_points.to_string().yellow()
    };

    println!(
        "{}",
        format!("║  You earned: {} points                   ║", player_color)
            .bright_cyan()
    );
    println!(
        "{}",
        format!("║  Computer earned: {} points             ║", computer_color)
            .bright_cyan()
    );

    println!("{}", "╚════════════════════════════════════════════╝".bright_cyan());

    if player_points > computer_points {
        println!("\n{}", ">> YOU WIN THIS ROUND! <<".bright_green().bold());
    } else if player_points < computer_points {
        println!("\n{}", ">> COMPUTER WINS THIS ROUND! <<".bright_red().bold());
    } else {
        println!("\n{}", ">> BOTH EARNED EQUALLY <<".yellow().bold());
    }

    thread::sleep(Duration::from_millis(1500));
}

fn display_game_summary(state: &GameState, stats: &Statistics) {
    clear_screen();

    println!("{}", "╔═══════════════════════════════════════════════════════════╗".bright_cyan());
    println!(
        "{}",
        "║                    GAME OVER                               ║".bright_cyan()
    );
    println!("{}", "╚═══════════════════════════════════════════════════════════╝".bright_cyan());
    println!();

    let final_diff = state.player_score - state.computer_score;
    println!(
        "{}",
        "═".repeat(60).bright_black()
    );

    let your_score_color = if state.player_score > state.computer_score {
        state.player_score.to_string().bright_green()
    } else if state.player_score < state.computer_score {
        state.player_score.to_string().bright_red()
    } else {
        state.player_score.to_string().yellow()
    };

    let computer_score_color = if state.computer_score > state.player_score {
        state.computer_score.to_string().bright_green()
    } else if state.computer_score < state.player_score {
        state.computer_score.to_string().bright_red()
    } else {
        state.computer_score.to_string().yellow()
    };

    println!(
        "  {} {}",
        "Your Final Score:".cyan().bold(),
        your_score_color
    );
    println!(
        "  {} {}",
        "Computer Final Score:".magenta().bold(),
        computer_score_color
    );
    println!(
        "  {} {}",
        "Score Differential:".yellow().bold(),
        if final_diff > 0 {
            format!("+{}", final_diff).bright_green()
        } else if final_diff < 0 {
            format!("{}", final_diff).bright_red()
        } else {
            "0".yellow()
        }
    );
    println!(
        "{}",
        "═".repeat(60).bright_black()
    );
    println!();

    if state.player_score > state.computer_score {
        println!("{}", "[WIN] VICTORY! YOU WON! [WIN]".bright_green().bold());
        println!();
        println!(
            "{}",
            "You outmaneuvered the computer and claimed victory!".green()
        );
    } else if state.player_score < state.computer_score {
        println!("{}", "[LOSS] DEFEAT! THE COMPUTER WON! [LOSS]".bright_red().bold());
        println!();
        println!(
            "{}",
            "The computer played a superior strategy this round.".red()
        );
    } else {
        println!("{}", "[TIE] IT'S A TIE! [TIE]".yellow().bold());
        println!();
        println!(
            "{}",
            "Both players fought to a draw!".yellow()
        );
    }

    println!();
    println!(
        "{}",
        format!("Total Rounds Played: {}", state.total_rounds).cyan()
    );
    println!(
        "{}",
        format!("Difficulty: {:?}", state.difficulty).yellow()
    );
    println!();
    println!("{}", "═".repeat(60).bright_black());
    println!("{}", "YOUR STATISTICS".yellow().bold());
    println!("{}", "═".repeat(60).bright_black());
    println!(
        "  {} {}",
        "Games Played:".cyan(),
        stats.games_played.to_string().bright_cyan()
    );
    println!(
        "  {} {} {} {} {}",
        "Record:".cyan(),
        format!("{} ", stats.games_won).bright_green().bold(),
        format!("W / {} ", stats.games_lost).bright_red().bold(),
        format!("L / {} ", stats.games_tied).yellow().bold(),
        "T".yellow()
    );
    println!(
        "  {} {:.1}%",
        "Win Rate:".cyan(),
        stats.win_rate().to_string().bright_cyan()
    );
    println!();
}

fn main_menu() -> u32 {
    print_title();
    println!("{}", "═".repeat(60).bright_black());
    println!("{}", "MAIN MENU".yellow().bold());
    println!("{}", "═".repeat(60).bright_black());
    println!();
    println!("  [1] [>] PLAY - Start a new game", );
    println!("  [2] [@] STATS - View your statistics");
    println!("  [3] [?] RULES - How to play");
    println!("  [4] [X] QUIT - Exit game");
    println!();

    loop {
        print!("{}: ", "Select an option (1-4)".cyan().bold());
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read line");

        match input.trim() {
            "1" => return 1,
            "2" => return 2,
            "3" => return 3,
            "4" => return 4,
            _ => {
                println!("{}", "[!] Invalid choice! Please enter 1-4.".red());
            }
        }
    }
}

fn display_rules() {
    print_title();
    print_payoff_matrix();
    println!();
    println!("{}", "GAME RULES & STRATEGY TIPS".yellow().bold());
    println!("{}", "═".repeat(60).bright_black());
    println!();
    println!("{}" ,"1. Each round, you and the computer choose to COOPERATE or DEFECT".cyan());
    println!("{}" ,"2. Your combined choices determine points earned this round".cyan());
    println!("{}" ,"3. The player with the highest score after all rounds WINS!".cyan());
    println!();
    println!("{}", "STRATEGIC TIPS:".bright_yellow().bold());
    println!("{}" ,"  + Cooperate for steady gains but risk being exploited".green());
    println!("{}" ,"  - Defect for short-term advantage but risk mutual punishment".red());
    println!("{}" ,"  * Pay attention to opponent patterns and adapt".magenta());
    println!("{}" ,"  ^ Mix strategies to keep opponent guessing".yellow());
    println!();
    println!("{}", "═".repeat(60).bright_black());
    print!("{}: ", "Press Enter to return to menu".cyan());
    io::stdout().flush().unwrap();
    let _ = io::stdin().read_line(&mut String::new());
}

fn display_stats(stats: &Statistics) {
    print_title();
    println!("{}", "═".repeat(60).bright_black());
    println!("{}", "YOUR GAME STATISTICS".yellow().bold());
    println!("{}", "═".repeat(60).bright_black());
    println!();

    if stats.games_played == 0 {
        println!("{}", "No games played yet. Start playing to build your statistics!".yellow());
    } else {
        println!(
            "  {} {}",
            "Total Games Played:".cyan().bold(),
            stats.games_played.to_string().bright_cyan()
        );
        println!(
            "  {} {}",
            "Games Won:".green().bold(),
            stats.games_won.to_string().bright_green()
        );
        println!(
            "  {} {}",
            "Games Lost:".red().bold(),
            stats.games_lost.to_string().bright_red()
        );
        println!(
            "  {} {}",
            "Games Tied:".yellow().bold(),
            stats.games_tied.to_string().bright_yellow()
        );
        println!(
            "  {} {:.1}%",
            "Win Rate:".magenta().bold(),
            stats.win_rate().to_string().bright_magenta()
        );
        println!(
            "  {} {}",
            "Total Points Earned:".cyan().bold(),
            stats.total_points.to_string().bright_cyan()
        );
        println!(
            "  {} {}",
            "Best Score Differential:".green().bold(),
            format!("+{}", stats.best_score_differential)
                .bright_green()
        );
        println!(
            "  {} {}",
            "Worst Score Differential:".red().bold(),
            format!("{}", stats.worst_score_differential).bright_red()
        );
    }

    println!();
    println!("{}", "═".repeat(60).bright_black());
    print!("{}: ", "Press Enter to return to menu".cyan());
    io::stdout().flush().unwrap();
    let _ = io::stdin().read_line(&mut String::new());
}

fn main() {
    loop {
        let choice = main_menu();

        match choice {
            1 => {
                // Play game
                let mut stats = Statistics::load();
                print_title();
                print_payoff_matrix();

                let difficulty = print_difficulty_menu();

                println!("{}","Excellent choice! Let's play!".bright_green().bold());
                println!();

                loop {
                    print!("{}: ", "How many rounds? (1-50)".cyan().bold());
                    io::stdout().flush().unwrap();

                    let mut input = String::new();
                    io::stdin()
                        .read_line(&mut input)
                        .expect("Failed to read line");

                    if let Ok(rounds) = input.trim().parse::<u32>() {
                        if rounds >= 1 && rounds <= 50 {
                            let mut state = GameState::new(rounds, difficulty);

                            for _ in 0..rounds {
                                state.round += 1;
                                clear_screen();
                                print_title();
                                print_game_state(&state);

                                let player_move = get_player_move();
                                let computer_move = get_computer_move(&state.history, difficulty);

                                let (player_points, computer_points) =
                                    state.calculate_payoff(player_move, computer_move);

                                animate_round_result(
                                    player_move,
                                    computer_move,
                                    player_points,
                                    computer_points,
                                );

                                state.player_score += player_points;
                                state.computer_score += computer_points;
                                state.history.push((player_move, computer_move));
                            }

                            let score_diff = state.player_score - state.computer_score;
                            stats.games_played += 1;
                            stats.total_points += state.player_score;

                            if state.player_score > state.computer_score {
                                stats.games_won += 1;
                            } else if state.player_score < state.computer_score {
                                stats.games_lost += 1;
                            } else {
                                stats.games_tied += 1;
                            }

                            stats.best_score_differential =
                                stats.best_score_differential.max(score_diff);
                            stats.worst_score_differential =
                                stats.worst_score_differential.min(score_diff);

                            stats.save();

                            display_game_summary(&state, &stats);

                            println!();
                            print!("{}: ", "Press Enter to continue".cyan());
                            io::stdout().flush().unwrap();
                            let _ = io::stdin().read_line(&mut String::new());

                            print!("{}: ", "Play again? (y/n)".cyan().bold());
                            io::stdout().flush().unwrap();

                            let mut play_again = String::new();
                            io::stdin()
                                .read_line(&mut play_again)
                                .expect("Failed to read line");

                            if play_again.trim().to_lowercase() != "y" {
                                break;
                            }
                        } else {
                            println!("{}", "Please enter a number between 1 and 50.".red());
                        }
                    } else {
                        println!("{}", "Invalid input. Please enter a number.".red());
                    }
                }
            }
            2 => {
                let stats = Statistics::load();
                display_stats(&stats);
            }
            3 => {
                display_rules();
            }
            4 => {
                println!();
                println!(
                    "{}",
                    "Thanks for playing! Goodbye!".bright_green().bold()
                );
                break;
            }
            _ => {}
        }
    }
}
