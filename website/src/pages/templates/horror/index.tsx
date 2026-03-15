import React from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import { templates } from '@site/src/data/templates';
import styles from '@site/src/components/TemplatePage/styles.module.css';

export default function HorrorTemplates() {
  const category = 'horror';
  const categoryTemplates = templates.filter(t => t.category === category);

  return (
    <Layout
      title="Horror 3D World Templates"
      description="Free AI-generated horror 3D world templates. Haunted houses, liminal spaces, and scary environments.">
      
      <div className={styles.hero}>
        <div className="container">
          <h1 className={styles.heroTitle}>Horror Templates</h1>
          <p className={styles.heroSubtitle}>
            Craft your nightmare. Terrifying environments, liminal backrooms, and haunted locations.
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
