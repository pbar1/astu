import type { ReactNode } from "react";
import clsx from "clsx";
import Heading from "@theme/Heading";
import styles from "./styles.module.css";

type FeatureItem = {
  title: string;
  Svg: React.ComponentType<React.ComponentProps<"svg">>;
  description: ReactNode;
};

const FeatureList: FeatureItem[] = [
  {
    title: "Large-Scale Execution",
    Svg: require("@site/static/img/astu_largescale.svg").default,
    description: (
      <>
        <code>astu</code> is built for running commands and network pings at
        scale. Resolve targets, authenticate, execute commands, and transfer
        files efficiently.
      </>
    ),
  },
  {
    title: "Automated Target Resolution",
    Svg: require("@site/static/img/astu_target.svg").default,
    description: (
      <>
        <code>astu</code> resolves hostnames, CIDR blocks, and more, ensuring
        seamless execution across dynamic environments.
      </>
    ),
  },
  {
    title: "Results & Aggregation",
    Svg: require("@site/static/img/astu_storage.svg").default,
    description: (
      <>
        Capture execution results, save them in persistent storage, and view
        trends with built-in aggregation capabilities.
      </>
    ),
  },
];

function Feature({ title, Svg, description }: FeatureItem) {
  return (
    <div className={clsx("col col--4")}>
      <div className="text--center">
        <Svg className={styles.featureSvg} role="img" />
      </div>
      <div className="text--center padding-horiz--md">
        <Heading as="h3">{title}</Heading>
        <p>{description}</p>
      </div>
    </div>
  );
}

export default function HomepageFeatures(): ReactNode {
  return (
    <section className={styles.features}>
      <div className="container">
        <div className="row">
          {FeatureList.map((props, idx) => (
            <Feature key={idx} {...props} />
          ))}
        </div>
      </div>
    </section>
  );
}
