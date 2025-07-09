import { useEffect, useState } from "react";
import type { Route } from "../+types/root";
import LetterGrid from "../components/LetterGrid";
import Scoreboard from "../components/Scoreboard";

interface LetterGridResponse {
    outer_letters: string[];
    center_letter: string;
}

export function meta({ }: Route.MetaArgs) {
    return [
        { title: "Spelling Gene" },
        { name: "description", content: "Gene guessing game" },
    ];
}

const App: React.FC = () => {
    const [letters, setLetters] = useState<string[]>([]);
    const [guessedSymbols, setGuessedSymbols] = useState<string[]>([]);
    const [currentGuess, setCurrentGuess] = useState("");
    const [score, setScore] = useState(0);

    const min_length = 4;
    const min_symbols = 10;
    const num_letters = 7;
    const seed = Math.floor(new Date().getTime() / (1000 * 60 * 60 * 24));

    const handleLetterClick = (letter: string) => {
        setCurrentGuess((prev) => prev + letter);
    };

    useEffect(() => {
        const handleKeyUp = (event: KeyboardEvent) => {
            const key = event.key.toUpperCase();

            if (key === 'BACKSPACE') {
                setCurrentGuess((prev) => prev.slice(0, -1));
            } else if (key === 'ENTER') {
                handleSubmit();
            } else if (key.length === 1 && /^[A-Z\-]$/.test(key)) {
                if (letters.includes(key)) {
                    handleLetterClick(key);
                }
            }
        };

        window.addEventListener("keyup", handleKeyUp);
        return () => window.removeEventListener("keyup", handleKeyUp);
    }, [letters, currentGuess]);

    const handleSubmit = async () => {
        const symbol = currentGuess.toUpperCase();
        if (symbol.length < 4 || guessedSymbols.includes(symbol)) {
            setCurrentGuess("");
            return;
        }

        const valid = async () => {
            try {
                const response = await fetch(`http://${process.env.REACT_APP_HOST}:${process.env.REACT_APP_PORT}/api/v1/spelling-gene-guess/${seed}/${min_length}/${min_symbols}/${num_letters}/${symbol}`, {
                    method: "GET",
                    headers: {
                        "Content-Type": "application/json",
                    }
                });

                if (!response.ok) {
                    return false;
                }

                const result: boolean = await response.json();
                return result
            } catch (error) {
                return false;
            }
        }

        if (await valid()) {
            setGuessedSymbols([...guessedSymbols, symbol]);
            setScore(score + symbol.length);
        }

        setCurrentGuess("");
    };

    const handleClear = () => {
        setCurrentGuess("");
    };

    useEffect(() => {
        const doFetch = async () => {
            const response = await fetch(`http://${process.env.REACT_APP_HOST}:${process.env.REACT_APP_PORT}/api/v1/spelling-gene/${seed}/${min_length}/${min_symbols}/${num_letters}`, {
                method: 'GET',
                headers: {
                    'Content-Type': 'application/json',
                }
            });

            if (!response.ok) {
                throw new Error(`API error: ${response.status} ${response.statusText}`);
            }

            const result: LetterGridResponse = await response.json();
            setLetters([result.center_letter, ...result.outer_letters]);
        };

        doFetch();
    }, []);

    return (
        <div className="min-h-screen bg-gray-100 flex flex-col items-center justify-center p-4 font-inter">
            <Scoreboard score={score} count={guessedSymbols.length} genes={guessedSymbols} />
            <LetterGrid letters={letters} onLetterClick={handleLetterClick} />
            <div className="guess-display">
                <h3 className={`text ${currentGuess ? "text-gray-950" : "text-gray-300"}`}>{currentGuess ? currentGuess : "Guess"}</h3>
                <button className="button text-gray-950" onClick={handleSubmit}>Submit</button>
                <button className="button text-gray-950" onClick={handleClear}>Clear</button>
            </div>
        </div>
    );
}

export default App;
