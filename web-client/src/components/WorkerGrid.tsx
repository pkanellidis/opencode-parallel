import type { Worker } from '../types';
import { WorkerCard } from './WorkerCard';

interface WorkerGridProps {
  workers: Worker[];
  onAnswerQuestion: (
    workerId: number,
    requestId: string,
    answers: string[]
  ) => void;
  onAnswerPermission: (
    workerId: number,
    requestId: string,
    reply: 'once' | 'always' | 'reject'
  ) => void;
}

export function WorkerGrid({
  workers,
  onAnswerQuestion,
  onAnswerPermission,
}: WorkerGridProps) {
  if (workers.length === 0) {
    return (
      <div className="worker-grid-empty">
        <p>No workers yet. Send a task to get started.</p>
      </div>
    );
  }

  return (
    <div className="worker-grid">
      {workers.map((worker) => (
        <WorkerCard
          key={worker.id}
          worker={worker}
          onAnswerQuestion={(requestId, answers) =>
            onAnswerQuestion(worker.id, requestId, answers)
          }
          onAnswerPermission={(requestId, reply) =>
            onAnswerPermission(worker.id, requestId, reply)
          }
        />
      ))}
    </div>
  );
}
