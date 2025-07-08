import "./Scoreboard.css";

export default function Scoreboard({ score, count, words }: { score: number, count: number, words: string[] }) {
    return (
        <div className="scoreboard-container">
            <div className="scoreboard">
                <p>Score: {score}</p>
                <p>Words: {count}</p>
            </div>
            <div className="word-list">
                <ul>
                    {words.map((word, idx) => (
                        <li key={idx}>{word}</li>
                    ))}
                </ul>
            </div>
        </div>
    );
}
