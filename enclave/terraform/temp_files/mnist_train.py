import tensorflow as tf
import numpy as np
import os
import json
from datetime import datetime

# Load MNIST dataset
(x_train, y_train), (x_test, y_test) = tf.keras.datasets.mnist.load_data()

# Normalize pixel values
x_train = x_train.astype('float32') / 255
x_test = x_test.astype('float32') / 255

# Create model
model = tf.keras.Sequential([
    tf.keras.layers.Flatten(input_shape=(28, 28)),
    tf.keras.layers.Dense(128, activation='relu'),
    tf.keras.layers.Dropout(0.2),
    tf.keras.layers.Dense(10, activation='softmax')
])

# Compile model
model.compile(optimizer='adam',
              loss='sparse_categorical_crossentropy',
              metrics=['accuracy'])

# Train model
history = model.fit(x_train, y_train, epochs=5, validation_split=0.2)

# Evaluate model
test_loss, test_acc = model.evaluate(x_test, y_test)
print(f'\nTest accuracy: {test_acc}')

# Save model
timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')
model.save(f'mnist_{timestamp}.h5')

# Save metrics
metrics = {
    'test_accuracy': float(test_acc),
    'test_loss': float(test_loss),
    'training_history': history.history,
    'timestamp': timestamp
}

with open(f'mnist_metrics_{timestamp}.json', 'w') as f:
    json.dump(metrics, f, indent=2)
