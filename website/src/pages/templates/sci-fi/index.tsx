import React from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import { templates } from '@site/src/data/templates';
import styles from '@site/src/components/TemplatePage/styles.module.css';

export default function SciFiTemplates() {
  const category = 'sci-fi';
  const categoryTemplates = templates.filter(t => t.category === category);

  return (
    <Layout
      title="Sci-Fi 3D World Templates"
      description="Free AI-generated sci-fi 3D world templates. Space stations, alien planets, and futuristic bases.">
      
      <div className={styles.hero}>
        <div className="container">
          <h1 className={styles.heroTitle}>Sci-Fi Templates</h1>
          <p className={styles.heroSubtitle}>
            Build the future. Procedural space stations, alien biomes, and futuristic cities.
          </p>
        </div>
      </div>

      <div className="container">
        <div className={styles.featuresSection} style={{ marginTop: '4rem' }}>
            <div className={styles.relatedGrid}>
                {categoryTemplates.map(t => (
                    <Link 
                        key={t.id} 
                        to={`/templates/${t.slug}`} 
                        className={styles.relatedCard}>
                        <h4>{t.title}</h4>
                        <p>{t.description}</p>
                    </Link>
                ))}
            </div>
        </div>
      </div>
    </Layout>
  );
}
