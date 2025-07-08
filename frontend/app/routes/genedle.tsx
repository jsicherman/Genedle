import React, { useState, useEffect, useCallback, type JSX } from 'react';
import type { Route } from '../+types/root';

type GuessState = 'correct' | 'present' | 'absent' | 'empty';

interface GuessRow {
  letters: { char: string; state: GuessState }[];
}

interface ValidGuess {
  is_correct: boolean;
  result: GuessState[];
}

type GuessResult =
  | { type: 'invalid'; data: string }
  | { type: 'valid'; data: ValidGuess };

const getLetterClass = (state: GuessState) => {
  switch (state) {
    case 'correct':
      return 'bg-green-500 text-white';
    case 'present':
      return 'bg-yellow-500 text-white';
    case 'absent':
      return 'bg-gray-500 text-white';
    case 'empty':
    default:
      return 'bg-gray-200 border border-gray-300 text-gray-800';
  }
};

const getKeyClass = (state: GuessState) => {
  switch (state) {
    case 'correct':
      return 'bg-green-500 text-white';
    case 'present':
      return 'bg-yellow-500 text-white';
    case 'absent':
      return 'bg-gray-500 text-white';
    case 'empty':
    default:
      return 'bg-gray-300 text-gray-900 hover:bg-gray-400';
  }
};

export function meta({ }: Route.MetaArgs) {
  return [
    { title: "Genedle" },
    { name: "description", content: "Gene guessing game" },
  ];
}

const App: React.FC = () => {
  const seed = Math.floor(new Date().getTime() / (1000 * 60 * 60 * 24));

  const [secretWord, setSecretWord] = useState(seed);
  const [wordLength, setWordLength] = useState(0);
  const [guesses, setGuesses] = useState<GuessRow[]>([]);
  const [currentGuess, setCurrentGuess] = useState('');
  const [turn, setTurn] = useState(0);
  const [isGameOver, setIsGameOver] = useState(false);
  const [isHardMode, setIsHardMode] = useState(true);
  const [isLoading, setIsLoading] = useState(false);
  const [keyboardColors, setKeyboardColors] = useState<Map<string, GuessState>>(new Map());
  const [message, setMessage] = useState<string | null>(null);

  const maxGuesses = 5;

  useEffect(() => {
    const doFetch = async () => {
      const response = await fetch(`http://127.0.0.1:3000/api/v1/genedle-letters/${secretWord}`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        }
      });

      if (!response.ok) {
        throw new Error(`API error: ${response.status} ${response.statusText}`);
      }

      const result: number = await response.json();
      setWordLength(result);
    };

    doFetch();
  }, [secretWord]);

  const checkGuess = useCallback(async (guess: string): Promise<{ guessRow: GuessRow; isCorrect: boolean }> => {
    const payload = {
      word: guess.split(''),
      session: secretWord,
      mode: isHardMode ? 'hard' : 'normal',
    };

    const response = await fetch('http://127.0.0.1:3000/api/v1/genedle-guess', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(payload),
    });

    if (!response.ok) {
      throw new Error(`API error: ${response.status} ${response.statusText}`);
    }

    const result: GuessResult = await response.json();

    if (result.type == 'invalid') {
      if (result.data === 'not_enough_letters') {
        throw new Error('Not enough letters');
      } else if (result.data === 'too_many_letters') {
        throw new Error('Too many letters');
      } else if (result.data === 'invalid_letter') {
        throw new Error('Invalid letter in guess');
      } else if (result.data === 'not_in_corpus') {
        throw new Error(`Invalid gene symbol`);
      } else if (result.data === 'internal_error') {
        throw new Error(`Internal error`);
      } else {
        throw new Error('Unknown');
      }
    } else if (result.type == 'valid') {
      const newGuessRow: GuessRow = {
        letters: result.data.result.map((letterFeedback, i) => ({
          char: guess.charAt(i),
          state: letterFeedback,
        })),
      };
      return { guessRow: newGuessRow, isCorrect: result.data.is_correct };
    }

    return result;
  }, [secretWord, isHardMode]);

  const renderToggle = useCallback(() => {
    return <div className="absolute top-4 right-4">
      <label htmlFor="hard-mode-toggle" className="flex items-center cursor-pointer">
        <span className="mr-2 text-gray-700">Hard Mode</span>
        <div className="relative">
          <input
            type="checkbox"
            id="hard-mode-toggle"
            className="sr-only"
            disabled={guesses.length > 0}
            checked={isHardMode}
            onChange={() => setIsHardMode(prevMode => !prevMode)}
          />
          <div className={`block w-14 h-8 rounded-full transition ${isHardMode ? (guesses.length > 0 ? 'bg-blue-400' : 'bg-blue-600') : (guesses.length > 0 ? 'bg-gray-400' : 'bg-gray-600')}`}></div>
          <div className={`dot absolute left-1 top-1 bg-white w-6 h-6 rounded-full transition transform ${isHardMode ? 'translate-x-full' : 'translate-x-0'}`}></div>
        </div>
      </label>
    </div>;
  }, [isHardMode, guesses]);

  const renderBoard = useCallback(() => {
    const boardRows: JSX.Element[] = [];

    guesses.forEach((guessRow, rowIndex) => {
      boardRows.push(
        <div key={`guess-${rowIndex}`} className="flex justify-center mb-1">
          {guessRow.letters.map((letter, colIndex) => (
            <div
              key={`guess-${rowIndex}-${colIndex}`}
              className={`w-14 h-14 md:w-16 md:h-16 flex items-center justify-center text-3xl font-bold rounded-md mx-0.5 transition-all duration-300 ease-in-out ${getLetterClass(letter.state)}`}
            >
              {letter.char}
            </div>
          ))}
        </div>
      );
    });

    if (turn < maxGuesses) {
      const currentGuessLetters = currentGuess.split('');
      const currentRowLetters: { char: string; state: GuessState }[] = [];

      for (let i = 0; i < wordLength; i++) {
        currentRowLetters.push({
          char: currentGuessLetters[i] || '',
          state: currentGuessLetters[i] ? 'empty' : 'empty',
        });
      }

      boardRows.push(
        <div key={`current-guess-${turn}`} className="flex justify-center mb-1">
          {currentRowLetters.map((letter, colIndex) => (
            <div
              key={`current-guess-${turn}-${colIndex}`}
              className={`w-14 h-14 md:w-16 md:h-16 flex items-center justify-center text-3xl font-bold rounded-md mx-0.5 transition-all duration-300 ease-in-out ${getLetterClass(letter.state)}`}
            >
              {letter.char}
            </div>
          ))}
        </div>
      );
    }

    for (let i = guesses.length; i < maxGuesses - 1; i++) {
      boardRows.push(
        <div key={`empty-${i}`} className="flex justify-center mb-1">
          {Array(wordLength).fill(null).map((_, colIndex) => (
            <div
              key={`empty-${i}-${colIndex}`}
              className={`w-14 h-14 md:w-16 md:h-16 flex items-center justify-center text-3xl font-bold rounded-md mx-0.5 ${getLetterClass('empty')}`}
            ></div>
          ))}
        </div>
      );
    }

    return <div className="mb-2">{boardRows}</div>;
  }, [guesses, currentGuess, wordLength]);

  const updateKeyboardColors = useCallback((guessRow: GuessRow) => {
    setKeyboardColors(prevColors => {
      const newColors = new Map(prevColors);
      guessRow.letters.forEach(({ char, state }) => {
        const currentKeyColor = newColors.get(char);
        if (state === 'correct') {
          newColors.set(char, 'correct');
        } else if (state === 'present' && currentKeyColor !== 'correct') {
          newColors.set(char, 'present');
        } else if (state === 'absent' && currentKeyColor !== 'correct' && currentKeyColor !== 'present') {
          newColors.set(char, 'absent');
        }
      });
      return newColors;
    });
  }, []);

  const handleKeyup = useCallback(async (e: KeyboardEvent) => {
    if (isGameOver || isLoading) return;

    const key = e.key.toUpperCase();

    if (key === 'ENTER') {
      setIsLoading(true);

      try {
        const { guessRow, isCorrect } = await checkGuess(currentGuess);

        setGuesses(prevGuesses => [...prevGuesses, guessRow]);
        updateKeyboardColors(guessRow);

        if (isCorrect) {
          setMessage('Geneius!');
          setIsGameOver(true);
        } else if (turn === maxGuesses - 1) {
          setMessage('Game over');
          setIsGameOver(true);
        } else {
          setMessage(null);
        }

        setCurrentGuess('');
        setTurn(prevTurn => prevTurn + 1);
      } catch (error) {
        if (error instanceof Error) {
          setMessage(error.message);
        }
      } finally {
        setIsLoading(false);
      }
    } else if (key === 'BACKSPACE') {
      setMessage(null);
      setCurrentGuess(prevGuess => prevGuess.slice(0, -1));
    } else if (/^[A-Za-z0-9-.]$/.test(key) && currentGuess.length < wordLength) {
      setMessage(null);
      setCurrentGuess(prevGuess => prevGuess + key);
    }
  }, [currentGuess, wordLength, isGameOver, isLoading, secretWord, turn, checkGuess, updateKeyboardColors]);

  useEffect(() => {
    window.addEventListener('keyup', handleKeyup);
    return () => window.removeEventListener('keyup', handleKeyup);
  }, [handleKeyup]);

  const handleVirtualKey = useCallback((key: string) => {
    handleKeyup({ key: key } as KeyboardEvent);
  }, [handleKeyup]);

  const keyboardLayout = [
    ['1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-'],
    ['Q', 'W', 'E', 'R', 'T', 'Y', 'U', 'I', 'O', 'P'],
    ['A', 'S', 'D', 'F', 'G', 'H', 'J', 'K', 'L'],
    ['ENTER', 'Z', 'X', 'C', 'V', 'B', 'N', 'M', 'BACKSPACE'],
  ];

  const renderKeyboard = () => (
    <div className="flex flex-col items-center w-full max-w-lg mx-auto">
      {keyboardLayout.map((row, rowIndex) => (
        <div key={`keyboard-row-${rowIndex}`} className="flex justify-center mb-2 w-full">
          {row.map(key => (
            <button
              key={key}
              onClick={() => handleVirtualKey(key)}
              disabled={isGameOver || isLoading}
              className={`
                flex-grow h-14 md:h-16 rounded-md mx-0.5 text-lg font-bold uppercase
                flex items-center justify-center
                ${key.length > 1 ? 'px-4 flex-grow-[1.5]' : 'w-10 md:w-12'}
                ${getKeyClass(keyboardColors.get(key) || 'empty')}
                transition-colors duration-200 ease-in-out
                disabled:opacity-70 disabled:cursor-not-allowed
              `}
            >
              {key === 'BACKSPACE' ? (
                <svg xmlns="http://www.w3.org/2000/svg" className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2M3 12l6.938 9H21a1 1 0 001-1V4a1 1 0 00-1-1h-11.062L3 12z" />
                </svg>
              ) : (
                key
              )}
            </button>
          ))}
        </div>
      ))}
    </div>
  );

  const resetGame = useCallback(() => {
    setSecretWord(seed);
    setGuesses([]);
    setCurrentGuess('');
    setTurn(0);
    setIsGameOver(false);
    setIsHardMode(true);
    setMessage(null);
    setIsLoading(false);
    setKeyboardColors(new Map());
  }, []);

  useEffect(() => {
    resetGame();
  }, [resetGame]);

  return (
    <div className="min-h-screen bg-gray-100 flex flex-col items-center justify-center p-4 font-inter">
      {renderToggle()}
      {renderBoard()}
      {message && (
        <div className="bg-blue-100 border border-blue-400 text-blue-700 px-4 py-3 mb-4 rounded-md shadow-md text-center text-lg">
          {message}
        </div>
      )}
      {renderKeyboard()}
      {isGameOver && (
        <button
          onClick={resetGame}
          className="mt-2 px-6 py-3 bg-blue-600 text-white text-xl font-bold rounded-lg shadow-lg hover:bg-blue-700 transition-all duration-300 ease-in-out transform hover:scale-105"
        >
          Play Again
        </button>
      )}
    </div>
  )
}

export default App;
