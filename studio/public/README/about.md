<h2 align="center">Network Medicine Platform</h2>
<p align="center">A network medicine platform with biomedical knowledge graph and graph neural network for drug repurposing and disease mechanism.</p>

<p align="center">
<img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/yjcyxky/biomedgps/publish-bin.yml
?label=Build Status">
<img src="https://img.shields.io/github/license/yjcyxky/biomedgps.svg?label=License" alt="License"> 
<a href="https://github.com/yjcyxky/biomedgps/releases">
<img alt="Latest Release" src="https://img.shields.io/github/release/yjcyxky/biomedgps.svg?label=Latest%20Release"/>
</a>
</p>

## Goal

Construct and integrate <b>knowledge graph</b>, <b>multi-omics data</b> and <b>deep learning models</b> to understand the molecular mechanisms of human diseases or predict known drugs for new indications (Drug Repurposing).

## Features

### <a href="https://drugs.3steps.cn/#/predict-explain/predict-model">Predict Drug/Target</a> Module

- [x] Predict known drugs for your queried disease (Drug Repurposing).
- [x] Predict new indications for your queried drug.
- [x] Understand the molecular mechanisms of human diseases.
- [x] Predict similar diseases for your queried disease.
- [x] Predict similar drugs for your queried drug.

<p></p>

### <a href="https://drugs.3steps.cn/#/predict-explain/knowledge-graph">Explain Your Results</a> Module

- [x] Knowledge graph studio for graph query, visualization and analysis.
- [x] Graph neural network for drug discovery, disease mechanism, biomarker screening and discovering response to toxicant exposure.
- [x] Support customized knowledge graph schema and data source.
- [x] Support customized graph neural network model.
- [x] Support customized omics datasets.
- [x] Integrated large language models (such as vicuna, rwkv, chatgpt etc. more details on [chat-publications](https://github.com/yjcyxky/chat-publications)) for answering questions.

<p></p>

## Demo

<div style="display: flex; justify-content: center; flex-direction: column; align-items: center; margin-bottom: 20px;">
<img src="https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/chatbot.png?raw=true" width="60%" />
<h3>Demo1: Ask questions with chatbot</h3>
</div>

<div style="display: flex; justify-content: center; flex-direction: column; align-items: center; margin-bottom: 20px;">
<img src="https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/disease-similarities.png?raw=true" width="60%" />
<h3>Demo2: Find similar diseases with your queried disease</h3>
</div>

<div style="display: flex; justify-content: center; flex-direction: column; align-items: center; margin-bottom: 20px;">
<img src="https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/drug-targets-genes.png?raw=true" width="60%" />
<h3>Demo3: Predict drugs and related genes for your queried disease</h3>
</div>

<div style="display: flex; justify-content: center; flex-direction: column; align-items: center; margin-bottom: 20px;">
<img src="https://github.com/yjcyxky/biomedgps-studio/blob/master/public/assets/path.png?raw=true" width="60%" />
<h3>Demo4: Find potential paths between two nodes</h3>
</div>

<p></p>

## Ecosystem

| Name                                                                              | Language | Description                                                                                                                               |
| :-------------------------------------------------------------------------------- | :------- | :---------------------------------------------------------------------------------------------------------------------------------------- |
| [BioMedGPS Data](https://github.com/yjcyxky/biomedgps-data)                       | Python   | For building the knowledge graph of BioMedGPS and training the graph neural network models.                                               |
| [Chat Publications](https://github.com/yjcyxky/chat-publications)                 | Python   | Ask questions and get answers from publications.                                                                                          |
| [BioMedical Knowledgebases](https://github.com/yjcyxky/biomedical-knowledgebases) | Markdown | A collection of biomedical knowledgebases, ontologies, datasets and publications etc.                                                     |
| [R Omics Utility](https://github.com/yjcyxky/r-omics-utils)                       | R        | Utilities for omics data with R. It will be part of biomedgps system and provide visulization and analysis functions of multi-omics data. |

## Official Website

[https://prophetdb.org](https://prophetdb.org)
