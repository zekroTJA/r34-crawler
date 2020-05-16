package workerpool

import (
	"sync"
)

type Job func(workerId int, params ...interface{}) interface{}

type WorkerPool struct {
	jobs    chan jobWrapper
	results chan interface{}
	wg      sync.WaitGroup
}

type jobWrapper struct {
	job    Job
	params []interface{}
}

func New(size int) *WorkerPool {
	w := &WorkerPool{
		jobs:    make(chan jobWrapper),
		results: make(chan interface{}),
	}

	for i := 0; i < size; i++ {
		go w.spawnWorker(i)
	}

	return w
}

func (w *WorkerPool) Push(job Job, params ...interface{}) {
	w.jobs <- jobWrapper{
		job:    job,
		params: params,
	}
}

func (w *WorkerPool) Close() {
	close(w.jobs)
}

func (w *WorkerPool) Results() <-chan interface{} {
	return w.results
}

func (w *WorkerPool) WaitBlocking() {
	w.wg.Wait()
}

func (w *WorkerPool) spawnWorker(id int) {
	for job := range w.jobs {
		if job.job != nil {
			w.wg.Add(1)
			w.results <- job.job(id, job.params...)
			w.wg.Done()
		}
	}
}
