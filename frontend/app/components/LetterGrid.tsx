import "./LetterGrid.css";

function getLetterPositionStyle(index: number, total: number) {
    const angle = (index / total) * 2 * Math.PI;
    const radius = 100;
    const x = radius * Math.cos(angle);
    const y = radius * Math.sin(angle);

    return {
        transform: `translate(-50%, -50%) translate(${x}px, ${y}px)`
    };
}

export default function LetterGrid({ letters, onLetterClick }: { letters: string[], onLetterClick: (letter: string) => void }) {
    if (!letters || letters.length < 1) return null;

    const center = letters[0];
    const outer = letters.slice(1);

    return (
        <div className="hex-container">
            <div
                className="center-letter"
                onClick={() => onLetterClick(center)}
                role="button"
            >
                {center}
            </div>
            {outer.map((char, index) => (
                <div
                    key={index}
                    className="outer-letter"
                    onClick={() => onLetterClick(char)}
                    style={getLetterPositionStyle(index, outer.length)}
                    role="button"
                >
                    {char}
                </div>
            ))}
        </div>
    );
}
