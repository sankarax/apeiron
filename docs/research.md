# Research experiments

## Predator metabolism

The recorded ecology experiments use recurrent controllers, an initial population of 1000, seeds 42, 123 and 456, and a 5000-tick horizon. The baseline uses predator metabolism scale 1, attack cost 1 and range bonus 2. The raw trajectories are in [`ecology_experiments.txt`](../benchmarks/ecology_experiments.txt).

| Configuration | Final herbivores (three seeds) | Final predators | Coexistence |
| --- | --- | --- | ---: |
| Baseline | 803, 866, 749 | 0, 0, 0 | 0/3 |
| Attack cost 0.1 | 732, 920, 492 | 0, 0, 0 | 0/3 |
| Metabolism scale 0.4 | 353, 417, 527 | 89, 38, 92 | 3/3 |
| Range bonus 8 | 577, 1000, 671 | 0, 0, 0 | 0/3 |
| Combined changes | 362, 7, 998 | 130, 133, 0 | 2/3 |
| Combined with zones and hazards | 713, 0, 0 | 0, 0, 0 | 0/3 |

Reducing predator metabolism alone produced coexistence in all three runs at the endpoint. Baseline predators disappeared by tick 1000. The combined environment caused complete extinction in two runs. These results do not demonstrate long-term stability or generalize to other seeds and settings.

## Local research agent

`backend/app/scientist.py` uses Ollama with `qwen3:8b`, temperature 0.2 and context size 16384. The agent can retrieve configuration, create experiments, run them, compare results and inspect lineage. Experiment runs are independent of the live world. Structured validation restricts factors, seeds, population and tick counts; the tool loop has bounded iterations. Research results are stored with their request and model metadata. Incomplete runs can also be archived.

The public notebook loads `frontend/public/research-demo.json`, a recorded run rather than a simulated response from a live LLM. The experiment compares predator metabolism scales 1 and 0.4 over the same three seeds, six runs and 30,000 total ticks. Baseline kills total 531; reduced metabolism kills total 1959. Mean final population falls from 806 to approximately 505.3 while predator coexistence rises from 0/3 to 3/3. Lower population alone does not establish lower stability. The generated report must be read alongside the measured data.

## Experimental limitations

Three seeds are insufficient for strong statistical inference. Endpoint coexistence is not a measure of long-term equilibrium. The runs do not isolate the benefit of recurrent memory from other architectural differences. Future work should use more seeds, longer horizons, confidence intervals and controlled ablations. The simulation is an experimental software model, not a calibrated ecological prediction tool.
