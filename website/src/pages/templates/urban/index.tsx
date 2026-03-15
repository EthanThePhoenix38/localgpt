import React from 'react';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';
import { templates } from '@site/src/data/templates';
import styles from '@site/src/components/TemplatePage/styles.module.css';

export default function UrbanTemplates() {
  const category = 'urban';
  const categoryTemplates = templates.filter(t => t.category === category);

  return (
    <Layout
      title="Urban 3D World Templates"
      description="Free AI-generated urban 3D world templates. Modern cities, cyberpunk metropolises, and street scenes.">
      
      <div className={styles.hero}>
        <div className="container">
          <h1 className={styles.heroTitle}>Urban Templates</h1>
          <p className={styles.heroSubtitle}>
            Build the modern world. Bustling cities, cyberpunk skylines, and realistic streets.
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
