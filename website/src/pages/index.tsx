import {useState} from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';

import styles from './index.module.css';

function InstallCommands() {
  const [copiedGen, setCopiedGen] = useState(false);
  const [copiedCli, setCopiedCli] = useState(false);

  const handleCopy = (cmd: string, setCopied: (v: boolean) => void) => {
    navigator.clipboard.writeText(cmd);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className={styles.installWrap}>
      <div className={styles.installRow}>
        <span className={styles.installLabel}>World Building</span>
        <div className={styles.installCmd} onClick={() => handleCopy('cargo install localgpt-gen', setCopiedGen)}>
          <code>cargo install localgpt-gen</code>
          <button className={styles.copyBtn}>{copiedGen ? 'Copied!' : 'Copy'}</button>
        </div>
      </div>
      <div className={styles.installRow}>
        <span className={styles.installLabel}>CLI Assistant</span>
        <div className={styles.installCmd} onClick={() => handleCopy('cargo install localgpt', setCopiedCli)}>
          <code>cargo install localgpt</code>
          <button className={styles.copyBtn}>{copiedCli ? 'Copied!' : 'Copy'}</button>
        </div>
      </div>
    </div>
  );
}

function HomepageHeader() {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--dark', styles.heroBanner)}>
      <div className="container">
        <div className={styles.heroLogos}>
          <img
            src="/logo/localgpt-icon.svg"
            alt={siteConfig.title}
            className={styles.heroIcon}
          />
          <img
            src="/logo/localgpt-gear.gif"
            alt={siteConfig.title}
            className={styles.heroLogo}
          />
        </div>
        <InstallCommands />
        <p className="hero__subtitle">
          Build explorable 3D worlds with natural language — geometry, materials, lighting, audio, and behaviors.
          <br />
          Open source, runs locally, powered by <a href="https://bevyengine.org/" target="_blank" rel="noopener noreferrer">Bevy</a>.
        </p>
        <div className={styles.buttons}>
          <Link
            className="button button--primary button--lg"
            to="/docs/gen">
            Start Building
          </Link>
          <Link
            className="button button--secondary button--lg"
            to="/docs/intro">
            Documentation
          </Link>
        </div>
      </div>
    </header>
  );
}

export default function Home(): JSX.Element {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title="Home"
      description="LocalGPT - Build explorable 3D worlds with natural language. Open source, runs locally, powered by Bevy.">
      <HomepageHeader />
      <main>
        <HomepageFeatures />
      </main>
    </Layout>
  );
}
