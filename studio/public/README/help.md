## Common Use Cases

### Step 1: Register and Login

To access the network medicine platform, you need to register and create an account or use google, microsoft, or github to login. You can click the "Sign In" button on the top right corner of the page to start the process.

<div style="display: flex; justify-content: center; flex-direction: column; align-items: center; margin-bottom: 20px;">
<img src="https://github.com/yjcyxky/biomedgps/blob/master/studio/public/README/images/step1-sign-in.png?raw=true" width="60%" />
</div>

### Step 2: Predict Interactions

Once you are logged in, you can start predicting interactions between genes, drugs, symptoms, and diseases. You can use the first tab "Predict Drug/Target" on the top of the page to enter the prediction page. You can pick up a module according to your interest, such as Disease module, Drug module, Gene module, and Symptom module. Each module has a set of parameters that you can set to predict interactions. For example, at the Disease module, you can set the "Prediction Type" to "Similar Diseases", keep the "Relation Type for Prediction" as default, and search a disease name, such as "breast cancer", and set the "Top K" to 10. Then click the "Apply Parameters" button to start the prediction. The result will be shown in the table left to the parameters. After you get the result, you can pick up some rows you are interested in and click the "Explain" button to load these rows to a knowledge graph to explain your prediction.

<div style="display: flex; justify-content: center; flex-direction: column; align-items: center; margin-bottom: 20px;">
<img src="https://github.com/yjcyxky/biomedgps/blob/master/studio/public/README/images/step2-predict-page.png?raw=true" width="60%" />
</div>

### Step 3: Explain Your Prediction

There are several operations you can do on the knowledge graph. You can right-click on a node, right-click on a edge, right-click on a canvas to do different operations for the nodes, edges, and graph respectively. If you would like to know more common operations, you can click the button with a question mark on the top of the page to see the help document. This button is on the right side of the "Upload / Query" button.

<div style="display: flex; justify-content: center; flex-direction: column; align-items: center; margin-bottom: 20px;">
<img src="https://github.com/yjcyxky/biomedgps/blob/master/studio/public/README/images/step3-explain.png?raw=true" width="60%" />
</div>

<div style="display: flex; justify-content: center; flex-direction: column; align-items: center; margin-bottom: 20px;">
<img src="https://github.com/yjcyxky/biomedgps/blob/master/studio/public/README/images/step3-explain-right-click-node.png?raw=true" width="60%" />
</div>

<div style="display: flex; justify-content: center; flex-direction: column; align-items: center; margin-bottom: 20px;">
<img src="https://github.com/yjcyxky/biomedgps/blob/master/studio/public/README/images/step3-explain-right-click-edge.png?raw=true" width="60%" />
</div>

<div style="display: flex; justify-content: center; flex-direction: column; align-items: center; margin-bottom: 20px;">
<img src="https://github.com/yjcyxky/biomedgps/blob/master/studio/public/README/images/step3-explain-common-operations.png?raw=true" width="60%" />
</div>

NOTE: Video Tutorial is coming soon!

## Scenario 1: Target Discovery for Breast Cancer

Company A is at the forefront of breast cancer therapeutic development. Their Target Discovery Department is on a quest to identify novel gene targets that could revolutionize treatment options. The team's mission is critical: to not only pinpoint a promising target gene but also to assess its viability in terms of druggability and market competition.

### How the Platform Helps

The network medicine platform offers an integrated solution for Company A's needs. By leveraging advanced algorithms and comprehensive biomedical databases, the platform facilitates:

#### Identification of Potential Targets

Utilizing cutting-edge network analysis, the platform identifies gene targets implicated in breast cancer, drawing from a vast array of genomic and proteomic data.

#### Druggability Analysis

The platform evaluates the identified targets for their druggability, considering factors such as the gene's structure, expression in breast tissue, and the availability of modulators.

#### Market Competition Insight

It provides an in-depth analysis of existing and in-development therapies targeting similar pathways, offering insights into patent landscapes, clinical trial stages, and market saturation.

#### Outcome

Company A leverages these insights to prioritize targets that are not only scientifically promising but also strategically viable, significantly accelerating their path to groundbreaking therapeutic discoveries.

## Scenario 2: Competitor Molecule Analysis for PD-L1 Blockers

Company B's Drug Discovery Team is on a mission to develop a groundbreaking therapeutic molecule that can effectively block PD-L1, a key protein involved in evading the immune response in various cancers. To ensure their development is competitive and innovative, they need comprehensive insights into existing and emerging PD-L1 inhibitors.

### How the Platform Helps

The network medicine platform serves as an invaluable tool for Company B by providing:

#### Competitor Molecule Discovery

Through an exhaustive search across global databases, the platform identifies both approved and investigational PD-L1 inhibitors, offering a complete view of the competitive landscape.

#### Detailed Molecule Profiles

For each competitor molecule, the platform provides detailed information, including clinical trial stages, patent statuses, biophysical properties, and reported therapeutic efficacy.

#### Comparative Analysis

It enables Company B to conduct side-by-side comparisons of their molecule against competitors, considering factors like mechanism of action, safety profiles, and clinical benefits.

#### Outcome

Armed with this comprehensive and nuanced understanding of the competitive landscape, Company B can strategically navigate their molecule development, focusing on innovation, differentiation, and addressing unmet needs in the PD-L1 blockade therapeutic area.

## Scenario 3: Drug Repurposing for Symptom Defined Diseases