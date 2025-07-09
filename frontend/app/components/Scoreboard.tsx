import "./Scoreboard.css";

export default function Scoreboard({ score, count, genes }: { score: number, count: number, genes: string[] }) {
    return (
        <div className="scoreboard-container">
            <div className="scoreboard">
                <p>Score: {score}</p>
                <p>Genes: {count}</p>
            </div>
            <div className="gene-list">
                <ul>
                    {genes.map((gene, idx) => (
                        <li key={idx}>{gene}</li>
                    ))}
                </ul>
            </div>
        </div>
    );
}
